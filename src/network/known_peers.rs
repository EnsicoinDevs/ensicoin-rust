use sled::Db;
use dirs::data_dir;
use utils::error::Error;

pub struct KnownPeers;

impl KnownPeers {
    fn open() -> Result<Db, Error> {
        let mut path = data_dir().unwrap();
        path.push("ensicoin-rust/");
        path.push("known_peers");
        Ok(sled::Db::open(path)?)
    }

    pub fn add_peer(&self, ip: String) -> Result<(), Error> {
        let db = KnownPeers::open()?;
        db.insert(ip, &[])?;

        db.flush()?;
        Ok(())
    }

    pub fn get_peers(&self) -> Result<Vec<String>, Error> {
        let db = KnownPeers::open()?;
        let iter = db.iter();
        let r = iter.map( |x| String::from_utf8(x.unwrap().0.to_vec()).unwrap() ).collect();
        Ok(r)
    }
    
    pub fn del_peer(&self, ip: String) -> Result<(), Error> {
        let db = KnownPeers::open()?;

        db.remove(ip)?;

        db.flush()?;
        Ok(())
    }
}
