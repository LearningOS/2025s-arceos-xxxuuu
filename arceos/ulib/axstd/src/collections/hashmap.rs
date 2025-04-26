use alloc::vec::Vec;
use arceos_api::modules::axhal::misc::random;

pub trait Hash {
    fn hash<H: Hasher>(&self, hasher: &mut H);
}

pub trait Hasher {
    fn write_u8(&mut self, value: u8);
    fn finish(&self) -> u64;
}

pub struct DefaultHasher {
    buf: Vec<u8>,
}

impl DefaultHasher {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }
}

impl Hasher for DefaultHasher {
    fn write_u8(&mut self, value: u8) {
        self.buf.push(value);
    }

    fn finish(&self) -> u64 {
        let p = 131;
        let mut hash = random() as u64;
        for c in self.buf.iter() {
            hash = hash * p + *c as u64;
        }
        hash
    }
}

impl Hash for alloc::string::String {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        for c in self.as_bytes() {
            hasher.write_u8(*c);
        }
    }
}

static CAPACITY: usize = 256;

pub struct HashMap<K, V> {
    table: Vec<Vec<(K, V)>>,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        let mut table = Vec::with_capacity(CAPACITY);
        for _ in 0..CAPACITY {
            table.push(Vec::new());
        }
        HashMap { table }
    }

    fn hash(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % CAPACITY
    }

    pub fn insert(&mut self, key: K, value: V) {
        let hash = self.hash(&key);
        self.table[hash].push((key, value));
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash(key);
        self.table[hash]
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.table.iter().flat_map(|bucket| bucket.iter())
    }
}
