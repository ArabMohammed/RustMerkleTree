use sha3::{Digest, Sha3_256, Sha3_256Core};
use std::path::Path;
use std::fs::* ;
use std::io::Write;
use rand::Rng;
use std::io::{BufReader, Read};
//This defines Hash as an alias for an array 
//of 32 unsigned 8-bit integers (u8). 
pub type Hash = [u8; 32]; 

#[derive(Debug)]
pub struct MerkleTree{
    nodes: Vec<Hash>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof(pub Vec<(bool, Hash)>); 
impl MerkleTree {
    pub fn num_nodes(depth : usize) -> usize {
        let res : usize = (2_u32.pow(depth as u32 +1)).try_into().unwrap();
        res
    }
    pub fn depth_at_index(index : usize) -> usize{
        let log_result = (index as f64).log2() as usize ;
        log_result

    }
    pub fn new(depth : usize, initial_leaf_value: Hash) -> Self{
        let num_nodes = Self::num_nodes(depth);
        let mut nodes = vec![initial_leaf_value;num_nodes];
        let mut initial_value_at_depth = vec![initial_leaf_value;depth+1];
        for i in (0..depth).rev(){
            initial_value_at_depth[i]=concat(
                &initial_value_at_depth[i+1],
                &initial_value_at_depth[i+1],
            );
        }
        for i in 0..num_nodes {
            let depth = Self::depth_at_index(i);
            nodes[i]= initial_value_at_depth[depth];
        }
        MerkleTree {nodes}
    }
    pub fn index_of_parent(index : usize) -> Option<usize>{
        if index != 0 {
            Some(index/2)
        }else{
            None
        }
    }
    pub fn index_of_sibling(index : usize) -> Result<usize,()> {
        let mut sibling : usize = index + 1 ;
        if index%2==1 {
            sibling = index - 1 ;
        }
        Ok(sibling)
    }
    pub fn set(&mut self, leaf_index: usize, value : Hash){
        let index = self.nodes.len() / 2 + leaf_index ;
        self.nodes[index]=value;
        Self::set_recursively(&mut self.nodes, index);
    }
    // represents a mutable reference to a slice of Hash elements
    pub fn set_recursively(nodes: &mut [Hash], index : usize){
        let Some(parent) = Self::index_of_parent(index) else {
            return;
        };
        let sibling = Self::index_of_sibling(index).unwrap();
        let is_left = index%2==0; 
        if is_left {
            nodes[parent] = concat(&nodes[index],&nodes[sibling]);
        }else{
            nodes[parent] = concat(&nodes[sibling],&nodes[index]);
        }

        Self::set_recursively(nodes, parent);
    }
    
    pub fn proof(&self, leaf_index:usize) -> MerkleProof {
        let mut proof = vec![];
        println!("welcome in proof function");
        let index = self.nodes.len()/2 + leaf_index ;
        println!("gotten index :  {}",index);
        Self::proof_recursive(&mut proof, &self.nodes, index);
        MerkleProof(proof)
    }
    
    pub fn proof_recursive(proof : &mut Vec<(bool, Hash)>, nodes : &[Hash], index : usize){
        let Some(parent) = Self::index_of_parent(index) else {
            return;
        };
        let sibling = Self::index_of_sibling(index).unwrap();
        let is_left = index %2 == 0 ;
        proof.push((is_left,nodes[sibling]));
        Self::proof_recursive(proof, nodes, parent);
    }

    pub fn verify(proof : &MerkleProof, leaf_value: Hash)-> Hash {
        let mut hash = leaf_value ;
        for (is_left, sibling) in proof.0.iter(){
            if *is_left{
                hash = concat(&hash,sibling);
            }else{
                hash = concat(sibling,&hash);
            }
        }
        hash
    }
    pub fn num_leaves(&self) -> usize {
        self.nodes.len()/2
    }

    pub fn root(&self) -> Hash {
        self.nodes[0]
    }
}
fn concat(a : &Hash, b:&Hash) -> Hash {
    let mut hasher = Sha3_256::new();
    hasher.update(a); 
    hasher.update(b);
    hasher.finalize().into() 
}
/*0
#[cfg(test)]
mod tests{
    use hex_literal::hex;
    use test_case::test_case;
    use super::*;
    #[test_case(5 => hex!(""))]
    #[test_case(7 => hex!(""))]
    #[test_case(20 => hex!(""))]
    fn initial_root(depth : usize) -> Hash {
        let initial_leaf = hex_literal::hex!("abababababababababababababababababababababababababababababababab");
        let tree = MerkleTree::new(depth,initial_leaf);
        
        let root = tree.root();

        println!("root = {}", hex::encode(root));

        root 
    }
    #[test]
    fn set_root(){
        const INITIAL : Hash = hex_literal::hex!("0000000000000000000000000000000000000000000000000000000000000000");
        let mut tree = MerkleTree::new(2,INITIAL);

        const NEW_VALUE : Hash = hex!("1111111111111111111111111111111111111111111111111111111111111111");
        tree.set(0,NEW_VALUE);
        let hash_1 = concat(&NEW_VALUE,&INITIAL);
        let hash_2 = concat(&NEW_VALUE,&INITIAL);
        let hash_0 = concat(&hash_1,&hash_2);

        assert_eq!(tree.root(),hash_0);
    }

    #[test]
    fn proof_of_first_leaf(){
        const INITIAL: Hash = hex_literal::hex!("0000000000000000000000000000000000000000000000000000000000000000");   
        let tree = MerkleTree::new(2,INITIAL);
        
        let initial_2 = concat(&INITIAL,&INITIAL);

        let proof = tree.proof(0);

        let manual_proof = MerkleProof(vec![(true, INITIAL),(true,initial_2)]);

        assert_eq!(proof, manual_proof);
    }

    #[test]
    fn proof_of_last_leaf(){
        const INITIAL: Hash = hex_literal::hex!("0000000000000000000000000000000000000000000000000000000000000000");   
        let tree = MerkleTree::new(2,INITIAL);
        
        let initial_2 = concat(&INITIAL,&INITIAL);

        let proof = tree.proof(3);

        let manual_proof = MerkleProof(vec![(false, INITIAL),(false,initial_2)]);

        assert_eq!(proof, manual_proof);
    }
    #[test]
    fn proof(){
        const INITIAL : Hash = hex_literal::hex!("0000000000000000000000000000000000000000000000000000000000000000");
        let leaves = [
            u32_to_hash(0),
            u32_to_hash(1),
            u32_to_hash(2),
            u32_to_hash(3),
        ];

        let mut tree = MerkleTree::new(2,INITIAL);
        for(i,leaf) in leaves.iter().enumerate(){
            tree.set(i,*leaf);
        }
        let proof = tree.proof(2);
        let siblings_1 = leaves[3];
        let siblings_2 = concat(&leaves[0],&leaves[1]);

        let manual_proof = MerkleProof(vec![(true,siblings_1),(false,siblings_2)]);
        assert_eq!(proof,manual_proof);   
    }

    #[test]
    fn proof_and_verify(){
        const INITIAL : Hash = hex_literal::hex!("0000000000000000000000000000000000000000000000000000000000000000");
        
        let mut tree = MerkleTree::new(10,INITIAL);
        for i  in 0..tree.num_leaves(){
            tree.set(i,u32_to_hash(i as u32));
        }
        let proof = tree.proof(tree.num_leaves()/2);
        let root = tree.root();
        let verify_root = MerkleTree::verify(&proof,u32_to_hash(tree.num_leaves() as u32 /2 ));

        assert_eq!(root,verify_root);
    }

    fn u32_to_hash(u : u32) -> Hash {
        let mut hash = [0u8; 32]; 
        hash[0..4].copy_from_slice(&u.to_le_bytes());
        
        hash
    }
}*/
/************************Function to hash files************************/
fn hash_file_sha3_256(file_path: &str) -> Result<Hash, Box<dyn std::error::Error>> {
    // Open the file
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    // Create a SHA3-256 hasher
    let mut hasher = Sha3_256::new();
    // Read the file in chunks and feed it to the hasher
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break; // EOF
        }
        hasher.update(&buffer[..bytes_read]);
    }
    // Finalize the hash and convert it to a hex string
    Ok(hasher.finalize().into())
   
}

fn main() -> Result<(),Box<dyn std::error::Error>> {
    /**Advantages of merkle trees 
     * 1) Inclusion proofs : verify efficiently that 
     * a certain leaf contributes to the root hash 
     * of the tree 
     * 2) Efficient updates : if we have 100 files 
     * and we need to have a hash for all of them , and for example
     * a file has been updated, then lest's suppose that each file has 
     * a hash representing a leaf in the merkle tree, then it will not 
     * be difficult to calculate the new hash, only depth+1 hashes will
     * be updated 
    */
    println!("Hello, world!");
    const INITIAL : Hash = hex_literal::hex!("0000000000000000000000000000000000000000000000000000000000000000");
    let mut tree: MerkleTree = MerkleTree::new(2,INITIAL);
    const NEW_VALUE : Hash = hex_literal::hex!("1111111111111111111111111111111111111111111111111111111111111111");
    //tree.set(0,NEW_VALUE);
    let hash_1 = concat(&INITIAL,&INITIAL);
    let hash_2 = concat(&INITIAL,&INITIAL);
    let hash_0 = concat(&hash_1,&hash_2);
    assert_eq!(tree.root(),hash_0);
    let result = tree.proof(2);
    println!("proof result : {:?}",result);
    /*********Create some files hash them and then store the hash in a mercury tree *****/
    let dir_path = Path::new("files");
    if !dir_path.exists() {
        // Create the directory and its parents if necessary
        std::fs::create_dir_all(dir_path);
    } else {
        println!("Directory already exists: {:?}", dir_path);
    }
    let mut rng = rand::thread_rng();
    let  depth : u32 = rng.gen_range(2..5); // Generates a number in the range [10, 99]
    let nb_files : usize = (2_u32.pow(depth as u32)).try_into().unwrap();
    // create a correspoding mercury tree 
    let mut files_tree: MerkleTree = MerkleTree::new(depth as usize,INITIAL);
    println!("depth of the tree {}",depth);
    for index in 1..nb_files+1 {
        // Specify the path to the file
        let mut filename  = format!("file{}",index);
        let file_path = dir_path.join(&filename);
        // Create a new file at the specified path or truncate it if it already exists
        let mut file = File::create(&file_path);
        // Define the content to write to the file
        let content = format!("welcome in file {}",index);
        // Write the content to the file
        let result = file.expect("Problem in writing file").write_all(content.as_bytes());
        // calculate the hash of the file and store it 
        let path_as_string: String = file_path.to_str().expect("Path is not valid UTF-8").to_string();
        let hash = match hash_file_sha3_256(&path_as_string) {
            Ok(hash) => hash,
            Err(e) => {return Err(e);}
        };
        // store hash in the position correspoding to that file 
        println!("==> set hash for file : {}",index-1);
        files_tree.set(index-1,hash);
    }   
    Ok(())
}
