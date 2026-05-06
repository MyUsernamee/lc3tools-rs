use std::fs::read;

use lc3tools_rs::LC3Simulator;

#[test]
fn test_obj_load() {
    let mut sim = LC3Simulator::new();
    let data = { read("./tests/lc3_programs/hello_world.obj").expect("Unable to read test .obj") };

    sim.load_obj(data, true).expect("Error loading test .obj");
    assert_eq!(sim.get_program_counter(), 0x3000);
}
