fn main() {
    cynic_codegen::register_schema("playerbrainz")
        .from_sdl_file("../../packages/playerbrainz/schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
