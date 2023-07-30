use crate::prelude::{Graph, GraphTensor, Shape, ShapeTracker, Tensor};
use itertools::Itertools;
use petgraph::stable_graph::NodeIndex;
use safetensors::tensor::{Dtype, View};
use std::collections::HashMap;
use std::{borrow::Cow, path::Path};

/// Tell luminal how to represent the module as a dict of (String, Tensor)'s
pub trait SerializeModule {
    fn serialize(&self, s: &mut Serializer);
}

/// A trait automatically derived on modules implementing SerializeModule, allowing access to `.save_to_file()`
pub trait SaveLoadModule: SerializeModule {
    /// Save the module to a file in the SafeTensors format
    fn save_to_file<P: AsRef<Path>>(&self, cx: &Graph, filename: P) {
        let tensors = self.state_dict(cx);
        safetensors::serialize_to_file(tensors, &None, filename.as_ref()).unwrap();
    }
    /// Get the state dict of the module
    fn state_dict<'a>(&'a self, cx: &'a Graph) -> HashMap<String, &'a Tensor> {
        let mut serializer = Serializer {
            current_path: "".to_string(),
            state: HashMap::default(),
        };
        self.serialize(&mut serializer);
        // Attempt to get all tensor data from the graph
        serializer
            .state
            .into_iter()
            .map(|(k, v)| (k, cx.get_tensor_ref(v).unwrap()))
            .collect()
    }
    /// Load module from state dict
    fn load_from_state_dict(&mut self, cx: &mut Graph, state_dict: HashMap<String, Tensor>) {
        let mut serializer = Serializer {
            current_path: "".to_string(),
            state: HashMap::default(),
        };
        self.serialize(&mut serializer);

        for (s, n) in serializer.state {
            cx.tensors.insert(n, (*state_dict.get(&s).unwrap()).clone());
        }
    }
    /// Load a module from a SafeTensors file
    fn load_from_file<P: AsRef<Path>>(&mut self, cx: &mut Graph, filename: P) {
        let data = std::fs::read(filename).unwrap();
        let st = safetensors::SafeTensors::deserialize(&data).unwrap();
        let state_dict = st
            .tensors()
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();
        self.load_from_state_dict(cx, state_dict);
    }
}

impl<T: SerializeModule> SaveLoadModule for T {}

pub struct Serializer {
    current_path: String,
    state: HashMap<String, NodeIndex>,
}

impl Serializer {
    pub fn tensor<S: Shape>(&mut self, name: &str, tensor: GraphTensor<S>) {
        self.state
            .insert(format!("{}/{}", self.current_path, name), tensor.id);
    }
    pub fn module<T: SerializeModule>(&mut self, name: &str, module: &T) {
        // Add new path component
        self.current_path.push('/');
        self.current_path.push_str(name);
        // Serialize
        module.serialize(self);
        // Remove new path component
        let mut components = self.current_path.split('/').collect_vec();
        components.pop();
        self.current_path = components.join("/");
    }
}

impl<'data> View for &'data Tensor {
    fn dtype(&self) -> Dtype {
        Dtype::F32 // For now just assume float, this should change in the future
    }
    fn shape(&self) -> &[usize] {
        self.shape.shape()
    }
    fn data(&self) -> Cow<[u8]> {
        self.data
            .as_any()
            .downcast_ref::<Vec<f32>>()
            .unwrap()
            .iter()
            .flat_map(|f| f.to_le_bytes().into_iter())
            .collect::<Vec<_>>()
            .into()
    }
    fn data_len(&self) -> usize {
        self.data.as_any().downcast_ref::<Vec<f32>>().unwrap().len()
    }
}

impl<'a> std::convert::From<safetensors::tensor::TensorView<'a>> for Tensor {
    fn from(value: safetensors::tensor::TensorView<'a>) -> Self {
        let chunked = value.data().chunks_exact(std::mem::size_of::<f32>());

        Tensor {
            data: Box::new(
                chunked
                    .map(|chunk| unsafe {
                        std::mem::transmute::<[u8; 4], f32>([
                            chunk[0], chunk[1], chunk[2], chunk[3],
                        ])
                    })
                    .collect::<Vec<f32>>(),
            ),
            shape: ShapeTracker::new(value.shape().to_vec()),
        }
    }
}
