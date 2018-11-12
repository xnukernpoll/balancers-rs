use super::*; 


use std::hash::{Hash, Hasher};

use p2c::*;
use futures::future;
use futures::future::JoinAll;


/*

struct CHash<T: Hash + Eq + Clone> {
  inner: Arc<RwLock < Vec<T> > >,
}







impl<T: Hash + Eq + Clone> Sharder for CHash<T> {
  type Item = T;
  type Key = u64;
  
  fn pick(&self, key: Self::Key) -> Option<Self::Item> {

    let inner = self.inner.read().unwrap();

    let l = inner.len() as i64;
    let i = jump_hash(key, l) as usize;
    
    match inner.iter().nth(i) {
      Some(r) => Some( r.clone() ),
      None => None
    }


    
  }


  fn pick_n(&self, key: Self::Key, n: usize) -> Vec<Self::Item>  {
    let inner = self.inner.read().unwrap();

    let l = inner.len() as i64;
    let i = jump_hash(key, l) as usize;

    circular::take_clockwise(&inner, i, n)
  }

}







*/




pub struct CHashLL<T: WeightedNode + Clone> {
  inner: MemberList<T>,
  factor: usize
}




impl <W: WeightedNode + Clone> From <Vec<W::Item>> for CHashLL<W> {
  
  fn from(v: Vec<W::Item>) -> CHashLL<W> {

    let loaded = v.into_iter().map(|inner| {
     W::new(inner)
    }).collect();
    
    let locked = RwLock::new(loaded);
    let inner = Arc::new(locked);
    CHashLL {inner, factor: 3} 
  }

  
}





impl <W: WeightedNode + Clone>  CHashLL<W> {

  
  fn pick(&self, key: u64) -> Option<W> {
    
    let inner = self.inner.read().unwrap();

    let l = inner.len() as i64;
    let i = jump_hash(key, l) as usize;

    let clock_wise = circular::take_clockwise(&inner, i, self.factor);

    let res = clock_wise.iter().min_by(|x, y|  x.load().cmp(&y.load() ) );
    res.map(|x| x.clone())
  }



  
  fn pick_n(&self, key: u64, n: usize) -> Vec<W> {

    let inner = self.inner.read().unwrap();

    let l = inner.len() as i64;
    let i = jump_hash(key, l) as usize;
    circular::take_clockwise(&inner, i, n)
  }



  pub fn dispatch<F, RV>(&self, key: u64, fun: F) -> Dispatch<W, RV> 
  where
    RV: Future + Sized,
    F: Fn(&W::Item) -> RV
  {

    let n = self.pick(key).unwrap();
    Dispatch::make(n.clone(), fun(&n.inner()) )

    
  }


  pub fn replicate<F, RV>(&self, key: u64, fun: F) -> JoinAll< Vec<Dispatch<W, RV>  > >
  where
    RV: Future + Sized,
    F: Fn(&W::Item) -> RV

  {


    let f = |x: W| {
      let n = x.inner(); 
      Dispatch::make(x, fun(&n) )
    };

    let replicas = self.pick_n(key, self.factor).into_iter().map( |x| f(x) ).collect();
    future::join_all(replicas)
  }


  

  
  
  
}













fn jump_hash(mut key: u64, num_buckets: i64) -> i64 {
    let mut b = 0;
    let mut j = 0;
    while j < num_buckets {
        b = j;
        key = key.wrapping_mul(2_862_933_555_777_941_757).wrapping_add(1);
        j = ((b.wrapping_add(1) as f64) * ((1i64 << 31) as f64)
            / ((key >> 33).wrapping_add(1) as f64)) as i64;
    }

    b
}





mod circular {



  pub fn take_clockwise<T: Clone>(v: &Vec<T>, i: usize, n: usize) -> Vec<T> {
    let max = i + n;
    let mut out = vec![];
    let size = v.len();
    
    for x in i..max {
      let r = &v[x % size];
      out.push(r.clone())
    }

    out
  }


  

            
}







#[cfg(test)]
mod tests {


  use super::*;
  #[test]
  fn test_circular() {
    let v: Vec<usize> = (1..6).collect();
    println!(" v's size is {}",v.len());
    
    let got = circular::take_clockwise(&v, 4, 3);
    let exp = vec![5, 1, 2];

    

    let (got1, exp1) = (
      circular::take_clockwise(&v, 0, 3),
      vec![1, 2, 3]
    );

    v[0];



    let (got2, exp2) = (
      circular::take_clockwise(&v, 2, 3),
      vec![3, 4, 5]
    ); 

    
    let (got3, exp3) = (
      circular::take_clockwise(&v, 3, 3),
      vec![4, 5, 1]
    );
    


    let (got3, exp3) = (
      circular::take_clockwise(&v, 4, 3),
      vec![5, 1, 2]
    );

    
    assert_eq!(exp, got);
    assert_eq!(got1, exp1);

    assert_eq!(got2, exp2);
    assert_eq!(got3, exp3);
    

  }


  

  
}