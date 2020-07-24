use rusqlite::{NO_PARAMS, Transaction};

pub fn create_tables(tr: &Transaction) -> rusqlite::Result<()> {
    tr.execute(
        "CREATE TABLE header (
            key TEXT,
            value TEXT
        )",
        NO_PARAMS,
    )?;
    tr.execute(
        "CREATE TABLE nodes (
            id INTEGER PRIMARY KEY,
            lat INTEGER NOT NULL,
            lon INTEGER NOT NULL
        )",
        NO_PARAMS,
    )?;
    tr.execute(
        "CREATE TABLE node_tags (
            node_id INTEGER,
            key TEXT,
            value TEXT,
            FOREIGN KEY(node_id) REFERENCES nodes(id)
        )",
        NO_PARAMS,
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
        NO_PARAMS,
    )?;
    tr.execute(
        "CREATE TABLE ways (
            id INTEGER PRIMARY KEY
        )",
        NO_PARAMS,
    )?;
    tr.execute(
        "CREATE TABLE way_tags (
            way_id INTEGER,
            key TEXT,
            value TEXT,
            FOREIGN KEY(way_id) REFERENCES ways(id)
        )",
        NO_PARAMS,
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
        NO_PARAMS,
    )?;
    tr.execute(
        "CREATE TABLE way_refs (
            way_id INTEGER,
            ref_node_id INTEGER,
            FOREIGN KEY(way_id) REFERENCES ways(id),
            FOREIGN KEY(ref_node_id) REFERENCES nodes(id) DEFERRABLE INITIALLY DEFERRED
        )", 
        NO_PARAMS,
    )?;
    tr.execute(
        "CREATE TABLE relations (
            id INTEGER PRIMARY KEY
        )",
        NO_PARAMS,
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
        NO_PARAMS,
    )?;
    tr.execute(
        "CREATE TABLE relation_tags (
            relation_id INTEGER,
            key TEXT,
            value TEXT,
            FOREIGN KEY(relation_id) REFERENCES relations(id)
        )",
        NO_PARAMS,
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
        NO_PARAMS,
    )?;
    Ok(())
}
