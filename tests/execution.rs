use std::{cell::RefCell, rc::Rc};

use lc3tools_rs::LC3Simulator;


#[test]
fn test_hello_world() {
    let test_output: Rc<RefCell<String>> = Rc::new(RefCell::new("".to_string()));

    let mut sim = LC3Simulator::with_os();
    sim.load_obj(include_bytes!("./lc3_programs/hello_world.obj").to_vec(), true).expect("Failed to load test file.");

    let value = test_output.clone();
    let cb = move |v| -> () {
        *(value.clone()).borrow_mut() += &String::from_utf8([v as u8].to_vec()).unwrap();
    };

    sim.add_write_callback(0xFE06, cb);
    sim.reset(); 

    while sim.step() {
        sim.write(0xFFFF, 0xFE04);
        dbg!(sim.get_memory()[0xFFFE]);
    }

    println!("{test_output:?}");
}
