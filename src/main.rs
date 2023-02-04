use rosm_pbf_reader::pbf;
use rosm_pbf_reader::util::*;
use rosm_pbf_reader::{read_blob, Block, BlockParser, DeltaValueReader, DenseNode, DenseNodeReader, TagReader};

use rusqlite::{params, Transaction};

use std::fs::File;

mod config;
use config::{read_config, Config, TableConfig};

mod db;

mod error;
use error::DumperError;

fn process_header_block(block: pbf::HeaderBlock, tr: &Transaction, config: &Config) -> rusqlite::Result<()> {
    if config.header.skip {
        return Ok(());
    }

    let mut insert_info = tr.prepare_cached("INSERT INTO header (key, value) VALUES (?1, ?2)")?;

    if let Some(bbox) = &block.bbox {
        insert_info.execute(params!["bbox_left", bbox.left])?;
        insert_info.execute(params!["bbox_right", bbox.right])?;
        insert_info.execute(params!["bbox_top", bbox.top])?;
        insert_info.execute(params!["bbox_bottom", bbox.bottom])?;
    }

    for feature in &block.required_features {
        insert_info.execute(params!["required_feature", feature])?;
    }

    for feature in &block.optional_features {
        insert_info.execute(params!["optional_feature", feature])?;
    }

    if let Some(writing_program) = &block.writingprogram {
        insert_info.execute(params!["writing_program", writing_program])?;
    }

    if let Some(source) = &block.source {
        insert_info.execute(params!["source", source])?;
    }

    if let Some(osmosis_replication_timestamp) = &block.osmosis_replication_timestamp {
        insert_info.execute(params!["osmosis_replication_timestamp", osmosis_replication_timestamp])?;
    }

    if let Some(osmosis_replication_sequence_number) = &block.osmosis_replication_sequence_number {
        insert_info.execute(params![
            "osmosis_replication_sequence_number",
            osmosis_replication_sequence_number
        ])?;
    }

    if let Some(osmosis_replication_base_url) = &block.osmosis_replication_base_url {
        insert_info.execute(params!["osmosis_replication_base_url", osmosis_replication_base_url])?;
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

fn insert_info<P: OsmPrimitive>(
    primitive: &P,
    block: &pbf::PrimitiveBlock,
    insert_stmt: &mut rusqlite::CachedStatement,
) -> rusqlite::Result<()> {
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

        insert_stmt.execute(params![
            primitive.id(),
            info.version,
            timestamp,
            info.uid,
            user,
            info.visible
        ])?;
    }
    Ok(())
}

fn process_primitive_block(
    block: pbf::PrimitiveBlock,
    config: &Config,
    stmts: &mut InsertStatements,
) -> rusqlite::Result<()> {
    let string_table = &block.stringtable;

    for group in &block.primitivegroup {
        if let Some(insert_node) = &mut stmts.node {
            if let Some(dense_nodes) = &group.dense {
                let nodes = DenseNodeReader::new(&dense_nodes, string_table);

                for node in nodes {
                    let coord = normalize_coord(node.lat, node.lon, &block);
                    insert_node.execute(params![node.id, coord.0, coord.1])?;

                    if let Some(insert_node_info) = &mut stmts.node_info {
                        insert_info(&node, &block, insert_node_info)?;
                    }

                    if let Some(insert_node_tag) = &mut stmts.node_tag {
                        for (key, value) in node.tags {
                            if !config.skip_tag_keys.contains(key?) {
                                insert_node_tag.execute(params![node.id, key?, value?])?;
                            }
                        }
                    }
                }
            } else {
                for node in &group.nodes {
                    let coord = normalize_coord(node.lat, node.lon, &block);
                    insert_node.execute(params![node.id, coord.0, coord.1])?;

                    if let Some(insert_node_tag) = &mut stmts.node_tag {
                        let tags = TagReader::new(&node.keys, &node.vals, string_table);

                        for (key, value) in tags {
                            if !config.skip_tag_keys.contains(key?) {
                                insert_node_tag.execute(params![node.id, key?, value?])?;
                            }
                        }
                    }

                    if let Some(insert_node_info) = &mut stmts.node_info {
                        insert_info(node, &block, insert_node_info)?;
                    }
                }
            }
        }

        if let Some(insert_way) = &mut stmts.way {
            for way in &group.ways {
                insert_way.execute(params![way.id])?;

                if let Some(insert_way_tag) = &mut stmts.way_tag {
                    let tags = TagReader::new(&way.keys, &way.vals, string_table);

                    for (key, value) in tags {
                        if !config.skip_tag_keys.contains(key?) {
                            insert_way_tag.execute(params![way.id, key?, value?])?;
                        }
                    }
                }

                if let Some(insert_way_info) = &mut stmts.way_info {
                    insert_info(way, &block, insert_way_info)?;
                }

                if let Some(insert_way_ref) = &mut stmts.way_ref {
                    let refs = DeltaValueReader::new(&way.refs);

                    for node_id in refs {
                        insert_way_ref.execute(params![way.id, node_id])?;
                    }
                }
            }
        }

        if let Some(insert_relation) = &mut stmts.relation {
            for relation in &group.relations {
                insert_relation.execute(params![relation.id])?;

                if let Some(insert_relation_tag) = &mut stmts.relation_tag {
                    let tags = TagReader::new(&relation.keys, &relation.vals, string_table);

                    for (key, value) in tags {
                        if !config.skip_tag_keys.contains(key?) {
                            insert_relation_tag.execute(params![relation.id, key?, value?])?;
                        }
                    }
                }

                if let Some(insert_relation_info) = &mut stmts.relation_info {
                    insert_info(relation, &block, insert_relation_info)?;
                }

                if let Some(insert_relation_member) = &mut stmts.relation_member {
                    let memids = DeltaValueReader::new(&relation.memids);

                    for (i, member_id) in memids.enumerate() {
                        let mut node_id = None;
                        let mut way_id = None;
                        let mut rel_id = None;

                        use pbf::relation::MemberType;

                        match MemberType::from_i32(relation.types[i]).expect("invalid MemberType enum") {
                            MemberType::Node => {
                                node_id = Some(member_id);
                            }
                            MemberType::Way => {
                                way_id = Some(member_id);
                            }
                            MemberType::Relation => {
                                rel_id = Some(member_id);
                            }
                        }

                        let string_id = relation.roles_sid[i];
                        let role = std::str::from_utf8(string_table.s[string_id as usize].as_ref())?;

                        insert_relation_member.execute(params![relation.id, node_id, way_id, rel_id, role])?;
                    }
                }
            }
        }
    }

    Ok(())
}

type Stmt<'a> = Option<rusqlite::CachedStatement<'a>>;

struct InsertStatements<'a> {
    node: Stmt<'a>,
    node_tag: Stmt<'a>,
    node_info: Stmt<'a>,

    way: Stmt<'a>,
    way_tag: Stmt<'a>,
    way_info: Stmt<'a>,
    way_ref: Stmt<'a>,

    relation: Stmt<'a>,
    relation_tag: Stmt<'a>,
    relation_info: Stmt<'a>,
    relation_member: Stmt<'a>,
}

fn prepare_insert_statements<'a>(tr: &'a Transaction, config: &Config) -> rusqlite::Result<InsertStatements<'a>> {
    let stmt = |sql: &str, table: &TableConfig, dependent_table: &TableConfig| {
        if !table.skip && !dependent_table.skip {
            tr.prepare_cached(sql).map(|statement| Some(statement))
        } else {
            Ok(None)
        }
    };

    Ok(InsertStatements {
        node: stmt("INSERT INTO nodes (id, lat, lon) VALUES (?1, ?2, ?3)", &config.nodes, &config.nodes)?,
        node_tag: stmt("INSERT INTO node_tags (node_id, key, value) VALUES (?1, ?2, ?3)", &config.node_tags, &config.nodes)?,
        node_info: stmt("INSERT INTO node_info (node_id, version, timestamp, user_id, user, visible) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", &config.node_info, &config.nodes)?,

        way: stmt("INSERT INTO ways (id) VALUES (?1)", &config.ways, &config.ways)?,
        way_tag: stmt("INSERT INTO way_tags (way_id, key, value) VALUES (?1, ?2, ?3)", &config.way_tags, &config.ways)?,
        way_info: stmt("INSERT INTO way_info (way_id, version, timestamp, user_id, user, visible) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", &config.way_info, &config.ways)?,
        way_ref: stmt("INSERT INTO way_refs (way_id, ref_node_id) VALUES (?1, ?2)", &config.way_refs, &config.ways)?,

        relation: stmt("INSERT INTO relations (id) VALUES (?1)", &config.relations, &config.relations)?,
        relation_tag: stmt("INSERT INTO relation_tags (relation_id, key, value) VALUES (?1, ?2, ?3)", &config.relation_tags, &config.relations)?,
        relation_info: stmt("INSERT INTO relation_info (relation_id, version, timestamp, user_id, user, visible) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", &config.relation_info, &config.relations)?,
        relation_member: stmt("INSERT INTO relation_members (relation_id, member_node_id, member_way_id, member_relation_id, role) VALUES (?1, ?2, ?3, ?4, ?5)", &config.relation_members, &config.relations)?,
    })
}

fn dump<Input: std::io::Read>(
    input_pbf: &mut Input,
    conn: &mut rusqlite::Connection,
    config: &Config,
) -> rusqlite::Result<()> {
    {
        let tr = conn.transaction()?;
        db::create_tables(&tr, config)?;
        tr.commit()?;
    }

    conn.execute("PRAGMA synchronous = OFF", [])?;
    conn.query_row_and_then("PRAGMA journal_mode = MEMORY", [], |_row| -> rusqlite::Result<()> {
        Ok(())
    })?;

    {
        let tr = conn.transaction()?;

        let mut stmts = prepare_insert_statements(&tr, config)?;

        let mut block_parser = BlockParser::default();

        while let Some(result) = read_blob(input_pbf) {
            match result {
                Ok(raw_block) => match block_parser.parse_block(raw_block) {
                    Ok(block) => match block {
                        Block::Header(header_block) => process_header_block(header_block, &tr, config)?,
                        Block::Primitive(primitive_block) => {
                            process_primitive_block(primitive_block, config, &mut stmts)?
                        }
                        Block::Unknown(unknown_block) => {
                            println!("Skipping unknown block of size {}", unknown_block.len())
                        }
                    },
                    Err(error) => println!("Error during parsing a block: {:?}", error),
                },
                Err(error) => println!("Error during reading the next blob: {:?}", error),
            }
        }

        drop(stmts); // Ensure `tr` is no longer used

        tr.commit()?;
    }

    Ok(())
}

fn main() -> Result<(), DumperError> {
    let config_path = std::env::args().nth(1).unwrap_or("config.json".to_string());
    let config = read_config(config_path)?;

    let mut input_pbf = File::open(&config.input_pbf)
        .map_err(|err| DumperError::new(err.into(), format!("Failed to open input PBF `{:?}`", config.input_pbf)))?;

    if config.overwrite_output && config.output_db.exists() {
        std::fs::remove_file(&config.output_db)
            .map_err(|err| DumperError::new(err.into(), format!("Failed to remove `{:?}`", config.output_db)))?;
    }

    let mut conn = rusqlite::Connection::open(&config.output_db).map_err(|err| {
        DumperError::new(
            err.into(),
            format!("Failed to open output SQLite database `{:?}`", config.output_db),
        )
    })?;

    dump(&mut input_pbf, &mut conn, &config)
        .map_err(|err| DumperError::new(err.into(), "An error occured during dumping".to_owned()))?;

    Ok(())
}
