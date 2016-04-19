// use katome::data::sequences::{sequence_to_u64, u64_to_sequence};
use ::data::input::{read_sequences};
use ::data::types::{Graph, Sequences, VecArc}; // , VecRcPtr};
// use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;

// use ::pbr::{ProgressBar};

// // this value is used only as a initiation dummy
// pub static mut VECTOR_RC: VecRcPtr = 0 as VecRcPtr;
// // this is only used to assure that the pointer to the Rc does not change throughout the program
// static mut rc_dummy: VecRcPtr = 0 as VecRcPtr;

pub fn make_it_happen(){
    let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
    // unsafe{
        // VECTOR_RC = &sequences as VecRcPtr;
    // }
    let mut graph: Graph = Graph::new();
    // read_sequences("***REMOVED***".to_string(),
    read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
                   sequences.clone(), &mut graph);
    print!("{}", sequences.borrow().len());
    // unsafe{
        // rc_dummy = &sequences as VecRcPtr;
    // }
    // assert!(unsafe{VECTOR_RC == rc_dummy}, "Pointer to Rc changed!");
}
