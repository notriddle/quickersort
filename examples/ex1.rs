extern crate quickersort;

fn main() {
    let mut ss = vec!["Introsort", "or", "introspective", "sort", "is",
                      "a", "hybrid", "sorting", "algorithm", "that",
                      "provides", "both", "fast", "average",
                      "performance", "and", "(asymptotically)", "optimal",
                      "worst-case", "performance"];
    quickersort::sort(&mut ss[..]);
    println!("alphabetically");
    for s in ss.iter() { println!("\t{}", s); }
    quickersort::sort_by(&mut ss[..], &|a, b| a.len().cmp(&b.len()));
    println!("\nby length");
    for s in ss.iter() { println!("\t{}", s); }
}
