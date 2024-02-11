# githuback
> quickly clone all your repos

## picture the scene...
your internet is about to drop out, your github account has been hacked, etc.

## what
- extremely fast repo downloading
- my public repos (66 at time of writing, most of which have substantial code in them) took around 10 secs
```bash
time cargo run --release -- --clone -p="werdl"
    Finished release [optimized] target(s) in 0.05s
     Running `target/release/githuback --clone -p=werdl`
⠁ Fetching repos (page 1)...                           
  Fetched repos                                        
  Fetched repos                                        
  Found 66 public repos for werdl
Cloning repos...
█████████████████████████████████████████████████ 66/66
real    0m9.492s
user    0m2.633s
sys     0m0.478s
```

## how
- calls to the github api (yes deals with paginating!) to get all your repos urls
- forms a list
- using tokio, clones every repo at the same time

## not working
- bear in mind only clones _public_ repos
- might require auth if you have a large number of repos and/or you use the tool freqently (pass in the github api key with arg `--auth_token` or `-a`)
### still no...
- please submit an issue with your host triple and/or `neofetch`/`screenfetch` output