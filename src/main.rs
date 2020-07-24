use rosm_pbf_reader::{PbfReader, Block, TagReader, DeltaValueReader, DenseNodeReader, DenseNode};
use rosm_pbf_reader::pbf;
use rosm_pbf_reader::util::*;

use rusqlite as sql;

use std::fs::File;

mod config;
use config::{Config, read_config};

mod error;
use error::DumperError;

fn create_tables(tr: &sql::Transaction) -> sql::Result<()> {
    tr.execute(
        "CREATE TABLE header (
            key TEXT,
            value TEXT
        )",
        sql::params![],
    )?;
    tr.execute(
        "CREATE TABLE nodes (
            id INTEGER PRIMARY KEY,
            lat INTEGER NOT NULL,
            lon INTEGER NOT NULL
        )",
        sql::params![],
    )?;
    tr.execute(
        "CREATE TABLE node_tags (
            node_id INTEGER,
            key TEXT,
            value TEXT,
            FOREIGN KEY(node_id) REFERENCES nodes(id)
        )",
        sql::params![],
    )?;
    tr.execute(
        "CREATE TABLE node_info (
            node_id INTEGER,
            version INTEGER,
            timestamp INTEGER,
            user_id INTEGER,
            user TEXT,
            visible BOOL,
            FOREIGN KEY(node_id) REFERENCES nodes(id)
        )",
        sql::params![],
    )?;
    tr.execute(
        "CREATE TABLE ways (
            id INTEGER PRIMARY KEY
        )",
        sql::params![],
    )?;
    tr.execute(
        "CREATE TABLE way_tags (
            way_id INTEGER,
            key TEXT,
            value TEXT,
            FOREIGN KEY(way_id) REFERENCES ways(id)
        )",
        sql::params![],
    )?;
    tr.execute(
        "CREATE TABLE way_info (
            way_id INTEGER,
            version INTEGER,
            timestamp INTEGER,
            user_id INTEGER,
            user TEXT,
            visible BOOL,
            FOREIGN KEY(way_id) REFERENCES ways(id)
        )",
        sql::params![],
    )?;
    tr.execute(
        "CREATE TABLE way_refs (
            way_id INTEGER,
            ref_node_id INTEGER,
            FOREIGN KEY(way_id) REFERENCES ways(id),
            FOREIGN KEY(ref_node_id) REFERENCES nodes(id) DEFERRABLE INITIALLY DEFERRED
        )", 
        sql::params![],
    )?;
    tr.execute(
        "CREATE TABLE relations (
            id INTEGER PRIMARY KEY
        )",
        sql::params![],
    )?;
    tr.execute(
        "CREATE TABLE relation_members (
            relation_id INTEGER,
            member_node_id INTEGER,
            member_way_id INTEGER,
            member_relation_id INTEGER,
            role TEXT,
            FOREIGN KEY(relation_id) REFERENCES relations(id)
        )", 
        sql::params![],
    )?;
    tr.execute(
        "CREATE TABLE relation_tags (
            relation_id INTEGER,
            key TEXT,
            value TEXT,
            FOREIGN KEY(relation_id) REFERENCES relations(id)
        )",
        sql::params![],
    )?;
    tr.execute(
        "CREATE TABLE relation_info (
            relation_id INTEGER,
            version INTEGER,
            timestamp INTEGER,
            user_id INTEGER,
            user TEXT,
            visible BOOL,
            FOREIGN KEY(relation_id) REFERENCES relations(id)
        )",
        sql::params![],
    )?;
    Ok(())
}

fn process_header_block(block: pbf::HeaderBlock, tr: &sql::Transaction) -> sql::Result<()> {
    let mut insert_info = tr.prepare_cached("INSERT INTO header (key, value) VALUES (?1, ?2)")?;

    if let Some(bbox) = &block.bbox {
        insert_info.execute(sql::params!["bbox_left", bbox.left])?;
        insert_info.execute(sql::params!["bbox_right", bbox.right])?;
        insert_info.execute(sql::params!["bbox_top", bbox.top])?;
        insert_info.execute(sql::params!["bbox_bottom", bbox.bottom])?;
    }

    for feature in &block.required_features {
        insert_info.execute(sql::params!["required_feature", feature])?;
    }

    for feature in &block.optional_features {
        insert_info.execute(sql::params!["optional_feature", feature])?;
    }

    if let Some(writing_program) = &block.writingprogram {
        insert_info.execute(sql::params!["writing_program", writing_program])?;
    }

    if let Some(source) = &block.source {
        insert_info.execute(sql::params!["source", source])?;
    }

    if let Some(osmosis_replication_timestamp) = &block.osmosis_replication_timestamp {
        insert_info.execute(sql::params!["osmosis_replication_timestamp", osmosis_replication_timestamp])?;
    }

    if let Some(osmosis_replication_sequence_number) = &block.osmosis_replication_sequence_number {
        insert_info.execute(sql::params!["osmosis_replication_sequence_number", osmosis_replication_sequence_number])?;
    }

    if let Some(osmosis_replication_base_url) = &block.osmosis_replication_base_url {
        insert_info.execute(sql::params!["osmosis_replication_base_url", osmosis_replication_base_url])?;
    }

    Ok(())
}

pub trait OsmPrimitive {
    fn id(&self) -> i64;
    fn info(&self) -> Option<&pbf::Info>;
}

impl OsmPrimitive for pbf::Node {
    fn id(&self) -> i64 {
        self.id
    }

    fn info(&self) -> Option<&pbf::Info> {
        self.info.as_ref()
    }
}

impl OsmPrimitive for pbf::Way {
    fn id(&self) -> i64 {
        self.id
    }

    fn info(&self) -> Option<&pbf::Info> {
        self.info.as_ref()
    }
}

impl OsmPrimitive for pbf::Relation {
    fn id(&self) -> i64 {
        self.id
    }

    fn info(&self) -> Option<&pbf::Info> {
        self.info.as_ref()
    }
}

impl<'a> OsmPrimitive for DenseNode<'a> {
    fn id(&self) -> i64 {
        self.id
    }

    fn info(&self) -> Option<&pbf::Info> {
        self.info.as_ref()
    }
}

fn insert_info<P: OsmPrimitive>(primitive: &P, block: &pbf::PrimitiveBlock, insert_stmt: &mut sql::CachedStatement) -> sql::Result<()> {
    if let Some(info) = primitive.info() {
        let user = if let Some(string_id) = info.user_sid {
            Some(std::str::from_utf8(block.stringtable.s[string_id as usize].as_ref()).unwrap())
        } else {
            None
        };

        let timestamp = if let Some(ts) = info.timestamp {
            Some(normalize_timestamp(ts, block))
        } else {
            None
        };

        insert_stmt.execute(sql::params![primitive.id(), info.version, timestamp, info.uid, user, info.visible])?;
    }
    Ok(())
}

fn process_primitive_block(block: pbf::PrimitiveBlock, tr: &sql::Transaction, config: &Config) -> sql::Result<()> {
    let mut insert_node = tr.prepare_cached("INSERT INTO nodes (id, lat, lon) VALUES (?1, ?2, ?3)")?;
    let mut insert_node_tag = tr.prepare_cached("INSERT INTO node_tags (node_id, key, value) VALUES (?1, ?2, ?3)")?;
    let mut insert_node_info = tr.prepare_cached("INSERT INTO node_info (node_id, version, timestamp, user_id, user, visible) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")?;

    let mut insert_way = tr.prepare_cached("INSERT INTO ways (id) VALUES (?1)")?;
    let mut insert_way_tag = tr.prepare_cached("INSERT INTO way_tags (way_id, key, value) VALUES (?1, ?2, ?3)")?;
    let mut insert_way_info = tr.prepare_cached("INSERT INTO way_info (way_id, version, timestamp, user_id, user, visible) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")?;
    let mut insert_way_ref = tr.prepare_cached("INSERT INTO way_refs (way_id, ref_node_id) VALUES (?1, ?2)")?;

    let mut insert_relation = tr.prepare_cached("INSERT INTO relations (id) VALUES (?1)")?;
    let mut insert_relation_tag = tr.prepare_cached("INSERT INTO relation_tags (relation_id, key, value) VALUES (?1, ?2, ?3)")?;
    let mut insert_relation_info = tr.prepare_cached("INSERT INTO relation_info (relation_id, version, timestamp, user_id, user, visible) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")?;
    let mut insert_relation_member = tr.prepare_cached("INSERT INTO relation_members (relation_id, member_node_id, member_way_id, member_relation_id, role) VALUES (?1, ?2, ?3, ?4, ?5)")?;

    let string_table = &block.stringtable;

    for group in &block.primitivegroup {
        if !config.skip_nodes {
            if let Some(dense_nodes) = &group.dense {
                let nodes = DenseNodeReader::new(&dense_nodes, string_table);

                for node in nodes {
                    let coord = normalize_coord(node.lat, node.lon, &block);
                    insert_node.execute(sql::params![node.id, coord.0, coord.1])?;

                    if !config.skip_node_info {
                        insert_info(&node, &block, &mut insert_node_info)?;
                    }

                    if !config.skip_node_tags {
                        for (key, value) in node.tags {
                            if !config.skip_tag_keys.contains(key?) {
                                insert_node_tag.execute(sql::params![node.id, key?, value?])?;
                            }
                        }
                    }
                }
            } else {
                for node in &group.nodes {
                    let coord = normalize_coord(node.lat, node.lon, &block);
                    insert_node.execute(sql::params![node.id, coord.0, coord.1])?;

                    if !config.skip_node_tags {
                        let tags = TagReader::new(&node.keys, &node.vals, string_table);

                        for (key, value) in tags {
                            if !config.skip_tag_keys.contains(key?) {
                                insert_node_tag.execute(sql::params![node.id, key?, value?])?;
                            }
                        }
                    }

                    if !config.skip_node_info {
                        insert_info(node, &block, &mut insert_node_info)?;
                    }
                }
            }
        }

        if !config.skip_ways {
            for way in &group.ways {
                insert_way.execute(sql::params![way.id])?;

                if !config.skip_way_tags {
                    let tags = TagReader::new(&way.keys, &way.vals, string_table);

                    for (key, value) in tags {
                        if !config.skip_tag_keys.contains(key?) {
                            insert_way_tag.execute(sql::params![way.id, key?, value?])?;
                        }
                    }
                }

                if !config.skip_way_info {
                    insert_info(way, &block, &mut insert_way_info)?;
                }

                if !config.skip_way_refs {
                    let refs = DeltaValueReader::new(&way.refs);

                    for node_id in refs {
                        insert_way_ref.execute(sql::params![way.id, node_id])?;
                    }
                }
            }
        }

        if !config.skip_relations {
            for relation in &group.relations {
                insert_relation.execute(sql::params![relation.id])?;

                if !config.skip_relation_tags {
                    let tags = TagReader::new(&relation.keys, &relation.vals, string_table);

                    for (key, value) in tags {
                        if !config.skip_tag_keys.contains(key?) {
                            insert_relation_tag.execute(sql::params![relation.id, key?, value?])?;
                        }
                    }
                }

                if !config.skip_relation_info {
                    insert_info(relation, &block, &mut insert_relation_info)?;
                }

                if !config.skip_relation_members {
                    let memids = DeltaValueReader::new(&relation.memids);

                    for (i, member_id) in memids.enumerate() {
                        let mut node_id = None;
                        let mut way_id = None;
                        let mut rel_id = None;

                        use pbf::mod_Relation::MemberType as MemberType;

                        match relation.types[i] {
                            MemberType::NODE => { node_id = Some(member_id); },
                            MemberType::WAY => { way_id = Some(member_id); },
                            MemberType::RELATION => { rel_id = Some(member_id); },
                        }

                        let string_id = relation.roles_sid[i];
                        let role = std::str::from_utf8(string_table.s[string_id as usize].as_ref())?;

                        insert_relation_member.execute(sql::params![relation.id, node_id, way_id, rel_id, role])?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn dump<Input: std::io::Read>(pbf_reader: &mut PbfReader<Input>, conn: &mut sql::Connection, config: &Config) -> sql::Result<()> {
    {
        let tr = conn.transaction()?;
        create_tables(&tr)?;
        tr.commit()?;
    }

    conn.execute("PRAGMA synchronous = OFF", sql::params![])?;
    conn.query_row_and_then("PRAGMA journal_mode = MEMORY", sql::params![], |_row| -> sql::Result<()> {
        Ok(())
    })?;

    {
        let tr = conn.transaction()?;

        while let Some(result) = pbf_reader.read_block() {
            match result {
                Ok(Block::Header(block)) => process_header_block(block, &tr)?,
                Ok(Block::Primitive(block)) => process_primitive_block(block, &tr, config)?,
                Ok(Block::Unknown(_)) => println!("Skipping unknown block"),
                Err(error) => println!("Error during read: {:?}", error),
            }
        }

        tr.commit()?;
    }

    Ok(())
}

fn main() -> Result<(), DumperError> {
    let config_path = std::env::args().nth(1).unwrap_or("config.json".to_string());
    let config = read_config(config_path)?;    

    let input_pbf = File::open(&config.input_pbf)
        .map_err(|err| DumperError::new(err.into(), format!("Failed to open input PBF `{:?}`", config.input_pbf)))?;

    let mut reader = PbfReader::new(input_pbf);

    if config.overwrite_output && config.output_db.exists() {
        std::fs::remove_file(&config.output_db)
            .map_err(|err| DumperError::new(err.into(), format!("Failed to remove `{:?}`", config.output_db)))?;
    }

    let mut conn = sql::Connection::open(&config.output_db)
        .map_err(|err| DumperError::new(err.into(), format!("Failed to open output SQLite database `{:?}`", config.output_db)))?;

    dump(&mut reader, &mut conn, &config)
        .map_err(|err| DumperError::new(err.into(), "An error occured during dumping".to_owned()))?;

    Ok(())
}
