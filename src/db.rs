use rusqlite::Transaction;

use super::config::{Config, TableConfig};

pub fn create_tables(tr: &Transaction, config: &Config) -> rusqlite::Result<()> {
    let create_index = |config: &TableConfig, table: &str| -> rusqlite::Result<()> {
        for columns in &config.create_index_on {
            let columns_split: Vec<&str> = columns.split(",").map(|c| c.trim()).collect();
            tr.execute(
                &format!(
                    "CREATE INDEX {}_{} ON {} ({})",
                    table,
                    columns_split.join("_"),
                    table,
                    columns_split.join(", ")
                ),
                [],
            )?;
        }
        Ok(())
    };

    if !config.header.skip {
        tr.execute(
            "CREATE TABLE header (
                key TEXT,
                value TEXT
            )",
            [],
        )?;

        create_index(&config.header, "header")?;
    }

    if !config.nodes.skip {
        tr.execute(
            "CREATE TABLE nodes (
                id INTEGER PRIMARY KEY,
                lat INTEGER NOT NULL,
                lon INTEGER NOT NULL
            )",
            [],
        )?;

        create_index(&config.nodes, "nodes")?;

        if !config.node_tags.skip {
            tr.execute(
                "CREATE TABLE node_tags (
                    node_id INTEGER,
                    key TEXT,
                    value TEXT,
                    FOREIGN KEY(node_id) REFERENCES nodes(id)
                )",
                [],
            )?;

            create_index(&config.node_tags, "node_tags")?;
        }

        if !config.node_info.skip {
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
                [],
            )?;

            create_index(&config.node_info, "node_info")?;
        }
    }

    if !config.ways.skip {
        tr.execute(
            "CREATE TABLE ways (
                id INTEGER PRIMARY KEY
            )",
            [],
        )?;

        create_index(&config.ways, "ways")?;

        if !config.way_tags.skip {
            tr.execute(
                "CREATE TABLE way_tags (
                    way_id INTEGER,
                    key TEXT,
                    value TEXT,
                    FOREIGN KEY(way_id) REFERENCES ways(id)
                )",
                [],
            )?;

            create_index(&config.way_tags, "way_tags")?;
        }

        if !config.way_info.skip {
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
                [],
            )?;

            create_index(&config.way_info, "way_info")?;
        }

        if !config.way_refs.skip {
            tr.execute(
                "CREATE TABLE way_refs (
                    way_id INTEGER,
                    ref_node_id INTEGER,
                    FOREIGN KEY(way_id) REFERENCES ways(id),
                    FOREIGN KEY(ref_node_id) REFERENCES nodes(id) DEFERRABLE INITIALLY DEFERRED
                )",
                [],
            )?;

            create_index(&config.way_refs, "way_refs")?;
        }
    }

    if !config.relations.skip {
        tr.execute(
            "CREATE TABLE relations (
                id INTEGER PRIMARY KEY
            )",
            [],
        )?;

        create_index(&config.relations, "relations")?;

        if !config.relation_members.skip {
            tr.execute(
                "CREATE TABLE relation_members (
                    relation_id INTEGER,
                    member_node_id INTEGER,
                    member_way_id INTEGER,
                    member_relation_id INTEGER,
                    role TEXT,
                    FOREIGN KEY(relation_id) REFERENCES relations(id)
                )",
                [],
            )?;

            create_index(&config.relation_members, "relation_members")?;
        }

        if !config.relation_tags.skip {
            tr.execute(
                "CREATE TABLE relation_tags (
                    relation_id INTEGER,
                    key TEXT,
                    value TEXT,
                    FOREIGN KEY(relation_id) REFERENCES relations(id)
                )",
                [],
            )?;

            create_index(&config.relation_tags, "relation_tags")?;
        }

        if !config.relation_info.skip {
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
                [],
            )?;

            create_index(&config.relation_info, "relation_info")?;
        }
    }

    Ok(())
}
