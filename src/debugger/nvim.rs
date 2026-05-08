use std::{cell::RefCell, path::Path, rc::Rc};

use nvim_oxi::{
    Dictionary, Function, String,
    api::{self, Error},
};

use crate::LC3Simulator;

#[nvim_oxi::plugin]
fn api() -> Dictionary {
    let sim: Rc<RefCell<Option<LC3Simulator>>> = Rc::new(RefCell::new(None));

    let create_sim: Function<String, Result<bool, api::Error>> =
        Function::from_fn(move |dir: String| -> Result<bool, api::Error> {
            if sim.borrow().is_some() {
                return Ok(false);
            }
            let mut sim = sim.borrow_mut();
            *sim = Some(LC3Simulator::with_os());
            let sim: &mut LC3Simulator = sim.as_mut().unwrap();
            let res = sim.load_obj(dir.as_bytes().to_vec(), true);

            Ok(res.is_ok())
        });

    todo!();
}
