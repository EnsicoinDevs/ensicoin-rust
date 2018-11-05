use model::block::Block as Block;
use std::rc::Rc;

pub struct Blockchain {
    chain : Vec<Rc<Block>>,
}

impl Blockchain {

    /**
     *  créer une nouvelle blockchain possédant un block (le block génésis)
     **/
    pub fn new(block : Block) -> Blockchain {
        return Blockchain {
            chain : vec![Rc::new(block)],
        }
    }

    /**
     *  renvoie une référence vers le dernier block de la blockchain
     **/
    pub fn get_latest_block(&self) -> &Block {
        match self.chain.last() {
            Some(block) => block,
            None => panic!()
        }
    }

    /**
     *  ajoute un block à la blockchain
     **/
    pub fn add_block(&mut self, block : Block) {
        if (block.previous_hash == self.get_latest_block().hash) && (block.hash == block.hash()) {
                self.chain.push(Rc::new(block));
                println!("Block valide");
        }
        else {
            println!("Block invalide");
        }
    }
}
