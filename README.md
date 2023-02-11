# rosm_pbf_sqlite_dumper

A simple command line tool for creating SQLite dumps from [OpenStreetMap PBF](https://wiki.openstreetmap.org/wiki/PBF_Format) files.

## Usage

The tool has a single, optional command line argument, which is the path to the configuration TOML file. The default value is `config.toml` .

## Configuration

The configuration is a TOML file, where the root object may contain the following keys:

- `input_pbf`: Path of the input PBF.
- `output_db`: Path of the output SQLite database.
- `overwrite_output`: If `true` and the given output file already exists, it'll be removed first. Default is `false`.
- `skip_tag_keys`: Array of node/way/relation tags which will be skipped.

For table-specific configuration the table's name should be used as the key, and the value must be a `TableConfiguration` object, containing:
- `skip`: If `true` the given table will be skipped. Default is `false`.
- `create_index_on`: Array of column list strings (columns separated by commas) to create indices for on the given table.

See `examples/config.toml` for an example configuration file.

## Output

The resulting SQLite database has the following tables (depending on configuration):

- `header`: Contents of the input PBF's header block, encoded as key/value pairs.
- `nodes`: Nodes, described by IDs and latitude/longitude pairs.
- `ways`: Ways, described by IDs.
  - `way_refs`: Nodes belonging to ways.
- `relations`: Relations, described by IDs.
  - `relation_members`: Nodes, ways, relations belonging to relations and their roles.

- `node/way/relation_tags`: Key/value pairs for nodes/ways/relations.
- `node/way/relation_info`: Other info for nodes/ways/relations (version, timestamp, user, etc.).
