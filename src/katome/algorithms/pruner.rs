use ::data::types::{Graph, VertexId, K_SIZE, VecArc};
use ::data::read_slice::ReadSlice;
use ::data::edges::Edges;

pub struct Pruner<'a> {
    graph: &'a mut Graph,
    vec: VecArc,
}

impl<'a> Pruner<'a> {
    pub fn new(graph_: &'a mut Graph, vec_: VecArc) -> Pruner {
        Pruner {
            graph: graph_,
            vec: vec_.clone(),
        }
    }

    pub fn prune_graph(&mut self) {
        self.prune_input();
    }

    fn prune_input(&mut self) {
        loop {
            let inputs = self.graph.iter()
                .filter(|&(_, val)| val.in_num == 0)
                .map(|(key, _)| key.offset)
                // FIXME change VertexId to ReadSlice
                .collect::<Vec<VertexId>>();
            println!("Num in: {}", inputs.len());
            let mut to_remove: Vec<ReadSlice> = vec![];
            for v in inputs {
                if let Some(dead_path) = check_input_vertex(self.graph, &v, self.vec.clone()) {
                    to_remove.extend(dead_path);
                }
            }
            if to_remove.is_empty() {
                return;
            }
            self.remove_in_path(to_remove.as_slice());
        }
    }

    fn remove_in_path(&mut self,  to_remove: &[ReadSlice]) {
        for v in to_remove.iter() {
            let val: Edges = self.graph.remove(v).unwrap();
            for o in val.outgoing.iter() {
                self.graph.get_mut(&ReadSlice::new(self.vec.clone(), o.0)).unwrap().in_num -= 1;
            }
        }
    }
}

fn check_input_vertex(graph: &Graph, vertex: &VertexId, vec: VecArc) -> Option<Vec<ReadSlice>> {
    let mut output_vec = vec![];
    let mut current_vertex: ReadSlice = ReadSlice::new(vec.clone(), *vertex);
    let mut cnt = 0;
    loop {
        let current_value = graph.get(&current_vertex).unwrap();
        if current_value.in_num > 1 {
            return Some(output_vec);
        }
        else if current_value.outgoing.len() == 0 {
            output_vec.push(current_vertex.clone());
            return Some(output_vec);
        }
        else {
            if cnt >= 2 * K_SIZE {
                return None;
            }
            cnt += 1;
            output_vec.push(current_vertex.clone());
            current_vertex = ReadSlice::new(vec.clone(), current_value.outgoing[0].0);
        }
    }
}
