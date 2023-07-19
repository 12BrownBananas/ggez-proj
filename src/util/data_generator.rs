use queues::IsQueue;
use slab_tree::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeStruct;
use serde::de::{self, Visitor, SeqAccess, MapAccess};
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use itertools::Itertools;
use std::collections::HashMap;
use std::ops;
use std::fs;
use std::path::Path;
use fraction::{Fraction, Decimal};
use queues;
use std::fmt;

const INPUT_FILE_NAME: &str = "rsc/data/difficulty_pools.json";
const DATA_FILES_LIST: &'static [&'static str] = &[INPUT_FILE_NAME];

/* #region Public Interface */
pub type TargetValidatorFunc = fn(f32) -> bool;
pub struct SetConfig {
    size: usize,
    target: String,
    validator: Option<TargetValidatorFunc>,
    difficulties: Vec<InputDifficulty>
}
impl SetConfig {
    pub fn new(size: usize, target_value: Option<Fraction>, validator_func: Option<TargetValidatorFunc>, difficulties: Vec<InputDifficulty>) -> SetConfig {
        let tar;
        match target_value {
            Some(val) => { tar = val.to_string() },
            None => {tar = "".to_string()}
        }
        return SetConfig {
            size: size,
            target: tar,
            validator: validator_func,
            difficulties: difficulties
        }
    }
}
#[derive(Debug)]
pub struct Board {
    pub input: Vec<i32>,
    pub target: Fraction,
    pub difficulty: InputDifficulty
}
impl Board {
    pub fn get_board_info(&self) -> String {
        format!("Input Vector: {:?}, Target Value: {:?}, Difficulty Rating: {:?}", self.input, self.target.to_string(), difficulty_to_string(self.difficulty))
    }
}
#[derive(Debug, Clone, Copy)]
pub enum InputDifficulty {
    Easy,
    Moderate,
    Hard
}
#[derive(Serialize, Deserialize, Clone)]
pub struct DifficultyPools {
    easy: Vec<Vec<i32>>,
    moderate: Vec<Vec<i32>>,
    hard: Vec<Vec<i32>>
}
impl DifficultyPools {
    fn new() -> DifficultyPools {
        DifficultyPools { easy: Vec::new(), moderate: Vec::new(), hard: Vec::new() }
    }
    fn get_closest_matching_populated_pool(&self, difficulty: InputDifficulty) -> Result<&Vec<Vec<i32>>, String> {
        let mut priority_vec: Vec<&Vec<Vec<i32>>> = Vec::new();
        match difficulty {
            InputDifficulty::Easy => {
                priority_vec.push(&self.easy);
                priority_vec.push(&self.moderate);
                priority_vec.push(&self.hard);
            },
            InputDifficulty::Moderate => {
                priority_vec.push(&self.moderate);
                priority_vec.push(&self.easy);
                priority_vec.push(&self.hard);
            },
            InputDifficulty::Hard => {
                priority_vec.push(&self.hard);
                priority_vec.push(&self.moderate);
                priority_vec.push(&self.easy);
            }
        }
        for pool in priority_vec {
            if pool.len() > 0 { return Ok(pool); }
        }
        return Err("All pools empty. Something's wrong!".to_string());
        
    }
    fn get_difficulty_of_pool(&self, pool: &Vec<Vec<i32>>) -> Result<InputDifficulty, String> {
        if pool == &self.easy { return Ok(InputDifficulty::Easy) }
        else if pool == &self.moderate {return Ok(InputDifficulty::Moderate)}
        else if pool == &self.hard { return Ok(InputDifficulty::Hard) }
        else { return Err("Supplied pool reference not bound to this pool.".to_string()) }
    }
}
pub fn value_is_integer(val: f32) -> bool {
    return val.floor() == val;
}
pub fn value_is_positive_integer(val: f32) -> bool {
    return value_is_integer(val) && val >= 0.0;
}
pub fn init(min_val: i32, max_val: i32, input_size: usize, force_regen: bool) {
    let mut safe_to_continue = true;
    /* Ensure output directories exist */
    match fs::create_dir_all("rsc/data") {
        Ok(_) => {},
        Err(_) => { safe_to_continue = Path::new("rsc").is_dir() && Path::new("rsc/data").is_dir() },
    }
    if !safe_to_continue { panic!("Data directory did not exist and could not be created. Terminating...") }
    if force_regen || !verify_data_integrity() { populate_output_directory(min_val, max_val+1, input_size) }
}
pub fn get_deserialized_input_data_pool_map() -> Result<HashMap<String, DifficultyPools>, String>  {
    if verify_data_integrity() == false { return Err("Data integrity could not be verified. Input data could not be deserialized.".to_string()); }
    let contents = std::fs::read_to_string(INPUT_FILE_NAME).expect("");
    let json_structure: HashMap<String, DifficultyPools> = serde_json::from_str(&contents).expect("");
    return Ok(json_structure);
}
pub fn get_set_of_inputs(mut pool_map: HashMap<String, DifficultyPools>, config: &SetConfig) -> Result<Vec<Board>, String> { //returns an error if the set could not be created to spec
    let mut result_vector = Vec::new();
    pool_map = remove_inputs_without_matching_difficulties(pool_map, &config.difficulties);
    while result_vector.len() < config.size {
        let &difficulty = config.difficulties.get(rand::random::<usize>()%config.difficulties.len()).expect("");
        let mut target = config.target.clone();
        if config.target == "" { 
            match get_random_viable_target(&pool_map, config.validator) {
                Ok(val) => {target = val},
                Err(e) => { return Err(e) } //Error implies this is impossible given input constraints, so we return
            }
        } 
        match get_input_of_difficulty_for_target(&pool_map, target.as_str(), difficulty) {
            Ok((input, index, pool_difficulty)) => {
                let map = pool_map.get_mut(target.as_str()).expect("");
                match pool_difficulty {
                    InputDifficulty::Easy => {
                        map.easy.remove(index);
                    },
                    InputDifficulty::Moderate => {
                        map.moderate.remove(index);
                    },
                    InputDifficulty::Hard => {
                        map.hard.remove(index);
                    }
                }
                result_vector.push(Board{ target: Fraction::from_str(&target).expect(&format!("Could not convert target {:?} to Fraction.", target)), input: input, difficulty: pool_difficulty } );
            },
            Err(e) => {
                //we found no viable inputs for the given target
                if config.target != "" { return Err(e); } //Since the target is fixed, this set has become impossible to create
                else { //Else it's probably still possible with some other target
                    pool_map.remove(&target); //So let's remove the target with no inputs
                    if pool_map.keys().len() <= 0 { return Err(e) } //And if that made our data set empty, then it means we cannot finish generating this input
                    //Note that this works because the error branch is only ever reached when NO pool is populated, irrespective of difficulty specifier.
                } 
            }
        }
    }
    Ok(result_vector)
}
/* #endregion */

/* #region Secret Inner-Workings */

#[derive(Deserialize, Serialize, Clone, Debug)]
enum OpType {
    Plus,
    Minus,
    Multiply,
    Divide,
    None
}
#[derive(Debug, Clone, Copy)]
struct SerializableFraction {
    fraction: Fraction,
}
impl Serialize for SerializableFraction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where 
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("SerializableFraction", 1)?;
        state.serialize_field("fraction", &self.fraction.to_string()).expect(&format!("Serialization of fraction {:?} failed", self.fraction));
        state.end()
    }
}
impl<'de> Deserialize<'de> for SerializableFraction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>, 
    {
        enum Field { Frac }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;
                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;
                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`fraction`")
                    }
                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "fraction" => Ok(Field::Frac),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }  
        struct SerializableFractionVisitor;
        impl<'de> Visitor<'de> for SerializableFractionVisitor {
            type Value = SerializableFraction;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct SerializableFraction")
            }
            fn visit_seq<V>(self, mut seq: V) -> Result<SerializableFraction, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let frac = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                Ok(SerializableFraction{ fraction: Fraction::from_str(frac).expect(&format!("Unable to deserialize fraction {:?}", frac)) })
            }
            fn visit_map<V>(self, mut map: V) -> Result<SerializableFraction, V::Error>
            where 
                V: MapAccess<'de>,
            {
                let mut frac = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Frac => {
                            if frac.is_some() {
                                return Err(de::Error::duplicate_field("fraction"));
                            }
                            frac = Some(map.next_value()?);
                        }
                    }
                }
                let frac = frac.ok_or_else(|| de::Error::missing_field("fraction"))?;
                Ok(SerializableFraction{ fraction: Fraction::from_str(frac).expect(&format!("Unable to deserialize fraction {:?}", frac)) })
            }
        }
        const FIELDS: &'static [&'static str] = &["fraction"];
        deserializer.deserialize_struct("SerializableFraction", FIELDS, SerializableFractionVisitor)
    }        
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OperationMarker {
    input_vector: Vec<SerializableFraction>,
    op_type: OpType,
}
impl OperationMarker {
    fn input_vector_to_i32(&self) -> Result<Vec<i32>, String> {
        let mut vec = Vec::new();
        for v in self.input_vector.as_slice() {
            let i = match Decimal::from_fraction(v.fraction).to_string().parse::<i32>() {
                Ok(i) => i,
                Err(_) => { return Err(format!("Could not convert input vector {:?} to vector of i32.", self.input_vector).to_string())}
            };
            vec.push(i);
        }
        return Ok(vec);
    }
    fn sum_input_vector(&self) -> Fraction {
        let mut acc = Fraction::from(0);
        if self.input_vector.len() <= 0 { return acc; };
        let mut sum_queue: queues::Queue<Fraction> = queues::Queue::new();
        for v in self.input_vector.as_slice() { sum_queue.add(v.fraction).expect(""); }
        
        while sum_queue.size() > 0 {
            acc = acc+sum_queue.remove().expect("");
        }
        return acc;
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct UnidirectionalNode {
    data: OperationMarker,
    children: Vec<UnidirectionalNode>
}
struct InputRanking {
    target: Fraction,
    difficulty: InputDifficulty,
    input: Vec<i32>
}

struct FractionalPair {
    first: Fraction,
    second: Fraction
}
#[derive(Debug)]
struct FrequencyCounter {
    map: HashMap<Fraction, u32> //value to occurrences
}
impl ops::Add<FrequencyCounter> for FrequencyCounter {
    type Output = FrequencyCounter;
    fn add(mut self, other: FrequencyCounter) -> FrequencyCounter {
        for k in other.map.keys() {
            match self.map.get_mut(k) {
                Some(entry) => {
                    match other.map.get(k) {
                        Some(other_entry) => {
                            *entry += *other_entry;
                        },
                        None => {}
                    }
                },
                None => {
                    self.map.entry(*k).or_insert(other.map.get(k).expect("Something went horribly wrong.").clone());
                }
            }
        }
        self
    }
}

fn verify_data_integrity() -> bool {
    for f in DATA_FILES_LIST {
        if !std::path::Path::new(f).exists() { return false }
    }
    return true;
}
fn difficulty_to_string(rating: InputDifficulty) -> String {
    match rating {
        InputDifficulty::Easy => {"Easy".to_string()},
        InputDifficulty::Moderate =>{"Moderate".to_string()},
        InputDifficulty::Hard =>{"Hard".to_string()}
    }
}
fn remove_inputs_without_matching_difficulties(pool_map: HashMap<String, DifficultyPools>, valid_difficulties: &Vec<InputDifficulty>) -> HashMap<String, DifficultyPools> {
    let mut new_pool_map = HashMap::new();
    for k in pool_map.keys() {
        let v: &DifficultyPools = pool_map.get(k).expect("");
        let mut add = false;
        for diff in valid_difficulties {
            match diff {
                InputDifficulty::Easy => { if v.easy.len() > 0 { add = true }},
                InputDifficulty::Moderate => {if v.moderate.len() > 0 { add = true }},
                InputDifficulty::Hard => {if v.hard.len() > 0 { add = true }}
            }
        }
        if add {
            new_pool_map.insert(k.clone(), v.clone());
        }
    }
    return new_pool_map;
}

fn get_random_viable_target(pool_map: &HashMap<String, DifficultyPools>, validator: Option<TargetValidatorFunc>) -> Result<String, String> {
    let mut keys: Vec<String> = pool_map.keys().cloned().collect();
    let mut idx = rand::random::<usize>()%keys.len();
    let local_keys = keys.clone();
    let mut k = local_keys.get(idx).expect("");
    let mut k_as_f32: f32 = Decimal::from_str(k).unwrap().to_string().parse::<f32>().unwrap(); //there might be a more efficient way to do this
    match validator {
        Some(v_func) => {
            while v_func(k_as_f32) == false {
                keys.remove(idx);
                idx = rand::random::<usize>()%&keys.as_slice().len();
                k = keys.get(idx).expect("");
                k_as_f32 = Decimal::from_str(k).unwrap().to_string().parse::<f32>().unwrap();
            }
        },
        None => {}
    }
    if keys.len() <= 0 { return Err("Ran out of viable targets...".to_string())}
    return Ok(k.to_string());
}

fn populate_output_directory(min_val: i32, max_val: i32, combination_count: usize) {
    /* Generate all possible outputs for all possible inputs */
    let root_nodes = generate_total_possibility_space(min_val, max_val, combination_count);
    let input_ranking_map = rank_all_inputs(&root_nodes);
    write_to_file(INPUT_FILE_NAME, "Failed to create difficulty pools file!", serde_json::to_string(&input_ranking_map).expect("Something went wrong when trying to write difficulty pools file").as_bytes());
}

fn get_input_of_difficulty_for_target(pool_map: &HashMap<String, DifficultyPools>, target: &str, difficulty: InputDifficulty) -> Result<(Vec<i32>, usize, InputDifficulty), String> {
    match pool_map.get(target) {
        Some(difficulty_pools) => {
            match difficulty_pools.get_closest_matching_populated_pool(difficulty) {
                Ok(pool) => {
                    let idx: usize = rand::random::<usize>()%(pool.len());
                    return Ok((pool.get(idx).expect("").clone(), idx, difficulty_pools.get_difficulty_of_pool(pool).expect("")));
                },
                Err(e) => {
                    return Err(e);
                }
            }
        },
        None => {
            return Err(format!("Could not find input of difficulty {:?} for target {:?}", difficulty, target)); 
        }
    }
}

fn write_to_file(path: &str, err_msg: &str, contents: &[u8]) {
    let mut f = File::create(path).expect(err_msg);
    let res = f.write_all(contents);
    match res {
        Err(_) => panic!("Failed to write file {:?}.", path),
        Ok(_) => {}
    }
}

fn rank_all_inputs(root_nodes: &Vec<UnidirectionalNode>) -> HashMap<String, DifficultyPools> {
    let mut ranked_inputs: HashMap<String, DifficultyPools> = HashMap::new();
    for r in root_nodes {
        let ranking = rank_input(r);
        for item in ranking {
            let item_name = &item.target.to_string();
            match ranked_inputs.get_mut(item_name) {
                Some(pool) => {
                    insert_input_into_pool(&item, pool)
                },
                None => {
                    let mut pool = DifficultyPools::new();
                    insert_input_into_pool(&item, &mut pool);
                    ranked_inputs.insert(item_name.clone(), pool);
                }
            }
        }
    }
    return ranked_inputs;
}

fn insert_input_into_pool(item: &InputRanking, pool: &mut DifficultyPools) {
    match item.difficulty {
        InputDifficulty::Easy => { pool.easy.push(item.input.clone()) },
        InputDifficulty::Moderate => { pool.moderate.push(item.input.clone()) },
        InputDifficulty::Hard => { pool.hard.push(item.input.clone()) }
    }
}

fn rank_input(input: &UnidirectionalNode) -> Vec<InputRanking> {
    let mut input_ranking = Vec::new();
    let leaf_value_map = count_leaf_instances_of(input);
    for k in leaf_value_map.map.keys() {
        let &occurrences = leaf_value_map.map.get(k).expect("");
        let mut difficulty = InputDifficulty::Easy;
        /* Note: Arbitrary values incoming */
        //Between 0 and 2 paths: "Hard" input
        //Between 2 and 5 paths: "Moderate" input
        //More paths than that: "Easy" input
        if occurrences > 0 && occurrences <= 2 {
            difficulty = InputDifficulty::Hard;
        }
        else if occurrences > 2 && occurrences <= 5 {
            difficulty = InputDifficulty::Moderate;
        }
        input_ranking.push(InputRanking{target: k.clone(), difficulty: difficulty, input: input.data.input_vector_to_i32().expect("Input ranking failed.") })
    }
    return input_ranking;
}

fn get_pairs(input_vector: &Vec<SerializableFraction>) -> Vec<FractionalPair> {
    let mut index = 0;
    let mut vector = Vec::new();
    while index < input_vector.len()-1 {
        let first = input_vector.get(index).unwrap().clone();
        let mut i = 0;
        while i < input_vector.len() {
            if i != index {
                let second = input_vector.get(i).unwrap().clone();
                vector.push(FractionalPair{ first: first.fraction, second: second.fraction });
            }
            i+=1;
        }
        index+=1;
    }
    return vector;
}
fn remove_pair(input_vector: &Vec<SerializableFraction>, pair: FractionalPair) -> Vec<SerializableFraction> {
    let mut vector = Vec::new();
    let mut values_to_remove = Vec::new();
    values_to_remove.push(pair.first);
    values_to_remove.push(pair.second);
    for &i in input_vector {
        if check_value_in_vec(values_to_remove.clone(), i.fraction) {
            values_to_remove.remove(values_to_remove.iter().position(|x| *x == i.fraction).unwrap());
        }
        else {
            vector.push(i);
        }
    }
    return vector;
}
fn check_value_in_vec<T: Clone + PartialEq>(v: Vec<T>, val: T) -> bool {
    let mut found = false;
    for i in v {
        if i == val { found = true }
    }
    return found;
}

fn count_leaf_instances_of(node: &UnidirectionalNode) -> FrequencyCounter {
    let mut map = FrequencyCounter{ map: HashMap::new() };
    for c in node.children.clone() {
        if c.children.len() <= 0 {
            let value = c.data.sum_input_vector();
            match map.map.get_mut(&value) {
                Some(entry) => {
                    *entry += 1;
                },
                None => {
                    map.map.entry(value).or_insert(1);
                }
            }
        }
        else { 
            let other_map = count_leaf_instances_of(&c);
            map = map+other_map;
        }    
    }
    return map;
}

fn generate_total_possibility_space(min_val: i32, max_val: i32, combination_count: usize) -> Vec<UnidirectionalNode> {
    let combinations = (min_val..max_val).combinations_with_replacement(combination_count);
    let mut root_nodes = Vec::new();
    for c in combinations {
        let input_vec: Vec<_> = c.iter().map(|i| *i).collect();
        root_nodes.push(get_serializable_root_node(input_vec));
    }
    return root_nodes;
}

fn get_serializable_root_node(input_vector: Vec<i32>) -> UnidirectionalNode {
    let tree = generate_tree_for_input(input_vector);
    
    let root = tree.root().unwrap().data();
    let children = tree.root().unwrap().children();
    let mut child_vec = Vec::new();
    for c in children {
        child_vec.push(UnidirectionalNode{ data: c.data().clone(), children: Vec::new() });
    }
    return UnidirectionalNode{ data: root.clone(), children: convert_child_nodes_to_json(&tree, tree.root().unwrap().node_id()) };
}

fn convert_child_nodes_to_json(tree: &Box<Tree<OperationMarker>>, root_id: NodeId) -> Vec<UnidirectionalNode> {
    let children = tree.get(root_id).unwrap().children();
    let mut child_vec = Vec::new();
    for c in children {
        child_vec.push(UnidirectionalNode{ data: c.data().clone(), children: convert_child_nodes_to_json(tree, c.node_id()) });
    }
    return child_vec;
}

fn vec_i32_to_fraction(vector: Vec<i32>) -> Vec<SerializableFraction> {
    let mut return_vec = Vec::new();
    for v in vector {
        return_vec.push(SerializableFraction { fraction: Fraction::from(v) });
    }
    return return_vec;
}

fn generate_tree_for_input(input_vector: Vec<i32>) -> Box<Tree<OperationMarker>> {
    let mut tree = Box::new(TreeBuilder::new().with_root(OperationMarker{ op_type: OpType::None, input_vector: vec_i32_to_fraction(input_vector) } ).build());
    let root_id = tree.root_id().expect("root does not exist");
    let mut root = tree.get_mut(root_id).unwrap();
    let input_vector = root.data().input_vector.clone();
    generate_decision_tree(&mut tree, root_id, &input_vector);
    return tree;
}

fn generate_decision_tree(tree: &mut Tree<OperationMarker>, node_id: NodeId, input_vector: &Vec<SerializableFraction>) {
    let pairs = get_pairs(&input_vector);
    for p in pairs {
        compute_operations(tree, node_id, p.first, p.second, remove_pair(&input_vector, p));
    }
}

fn compute_operations(tree: &mut Tree<OperationMarker>, node_id: NodeId, first_input: Fraction, second_input: Fraction, remaining_inputs: Vec<SerializableFraction>) {
    let mut vector = Vec::new();
    vector.push(compute_single_op(tree, node_id, first_input, second_input, remaining_inputs.clone(), compute_plus, OpType::Plus));
    vector.push(compute_single_op(tree, node_id, first_input, second_input, remaining_inputs.clone(), compute_minus, OpType::Minus));
    vector.push(compute_single_op(tree, node_id, first_input, second_input, remaining_inputs.clone(), compute_times, OpType::Multiply));
    vector.push(compute_single_op(tree, node_id, first_input, second_input, remaining_inputs.clone(), compute_divided_by, OpType::Divide));
    if remaining_inputs.len() > 0 {
        for v in vector {
            generate_decision_tree(tree, v.0, &v.1);
        }
    }
}
type Operation = fn(Fraction, Fraction) -> SerializableFraction;
fn compute_single_op<'a>(tree: &'a mut Tree<OperationMarker>, node_id: NodeId, first_input: Fraction, second_input: Fraction, remaining_inputs: Vec<SerializableFraction>, op_function: Operation, op_type: OpType) -> (NodeId, Vec<SerializableFraction>) {
    let mut vector = Vec::new();
    let mut node = tree.get_mut(node_id).unwrap();
    let mut r = remaining_inputs;
    vector.push(op_function(first_input, second_input));
    vector.append(&mut r);
    let mut child = node.append(OperationMarker{ op_type, input_vector: vector });
    return (child.node_id(), child.data().input_vector.clone());
}

fn compute_plus(first_input: Fraction, second_input: Fraction) -> SerializableFraction {
    SerializableFraction{ fraction: first_input+second_input }
}
fn compute_minus(first_input: Fraction, second_input: Fraction) -> SerializableFraction {
    SerializableFraction{ fraction: first_input-second_input }
}
fn compute_times(first_input: Fraction, second_input: Fraction) ->  SerializableFraction {
    SerializableFraction{ fraction: first_input*second_input }
}
fn compute_divided_by(first_input: Fraction, second_input: Fraction) -> SerializableFraction {
    SerializableFraction{ fraction: first_input/second_input }
}

/* #endregion */