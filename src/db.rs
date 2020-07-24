use rusqlite::{NO_PARAMS, Transaction};

use super::config::Config;

pub fn create_tables(tr: &Transaction, config: &Config) -> rusqlite::Result<()> {
    if !config.header.skip {
        tr.execute(
            "CREATE TABLE header (
                key TEXT,
                value TEXT
            )",
            NO_PARAMS,
        )?;
    }

    if !config.nodes.skip {
        tr.execute(
            "CREATE TABLE nodes (
                id INTEGER PRIMARY KEY,
                lat INTEGER NOT NULL,
                lon INTEGER NOT NULL
            )",
            NO_PARAMS,
        )?;

        if !config.node_tags.skip {
            tr.execute(
                "CREATE TABLE node_tags (
                    node_id INTEGER,
                    key TEXT,
                    value TEXT,
                    FOREIGN KEY(node_id) REFERENCES nodes(id)
                )",
                NO_PARAMS,
            )?;
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
                NO_PARAMS,
            )?;
        }
    }

    if !config.ways.skip {
        tr.execute(
            "CREATE TABLE ways (
                id INTEGER PRIMARY KEY
            )",
            NO_PARAMS,
        )?;

        if !config.way_tags.skip {
            tr.execute(
                "CREATE TABLE way_tags (
                    way_id INTEGER,
                    key TEXT,
                    value TEXT,
                    FOREIGN KEY(way_id) REFERENCES ways(id)
                )",
                NO_PARAMS,
            )?;
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
                NO_PARAMS,
            )?;
        }

        if !config.way_refs.skip {
            tr.execute(
                "CREATE TABLE way_refs (
                    way_id INTEGER,
                    ref_node_id INTEGER,
                    FOREIGN KEY(way_id) REFERENCES ways(id),
                    FOREIGN KEY(ref_node_id) REFERENCES nodes(id) DEFERRABLE INITIALLY DEFERRED
                )", 
                NO_PARAMS,
            )?;
        }
    }

    if !config.relations.skip {
        tr.execute(
            "CREATE TABLE relations (
                id INTEGER PRIMARY KEY
            )",
            NO_PARAMS,
        )?;

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
                NO_PARAMS,
            )?;
        }

        if !config.relation_tags.skip {
            tr.execute(
                "CREATE TABLE relation_tags (
                    relation_id INTEGER,
                    key TEXT,
                    value TEXT,
                    FOREIGN KEY(relation_id) REFERENCES relations(id)
                )",
                NO_PARAMS,
            )?;
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
                NO_PARAMS,
            )?;
        }
    }
    
    Ok(())
}
