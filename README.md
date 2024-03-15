  

# HeuristicFuzz (Rust)

Lets say you have a query set of terms, `q` and a reference database `r`. You want to associate all in `q` with `r` but lets say `q` is of low quality having for example incorrect names, concatenations etc.

  

### The heuristic approach
We could fuzzy search every term in `q` in `r` but having partial or bad matches does not mean a bad mapping, it might simply be a bad formatted query. For example lets say we have a query like "A yellow banana from the store" and want to match it to a taxonomy database having "banana". A fuzzy search will say "A yellow banana from the store" is only a `~6/20` match with "banana". We can solve this using a partial ratio match, this would say "A yellow _banana_ from the store" is is `100/100` match with "banana". However, **partial ratio mapping will be infeasibly slow for big datasets**.

Since partial mapping is too slow we have to come up with a heuristic approach to make an initial selection before running a fuzzy search. We first calculate all the shared bigrams between all `q x r`, lets say `Cov` Just shared bigrams are not accurate enough as very long texts might have all bigrams. For example, if we have the query "yellow banana" and we can match "yellow banana" and "yellow and browns bananas from a store". Both would match all bigrams from the query yet "yellow banana" seems closer to our query. To factor this in we also calculate the length difference for all `q x r`, lets say `Ldiff`. Each query is linked to a heap that stores the top matches, first based on `Cov` and if coverage is the same based on `Ldiff` being the smallest. This is already pretty accurate, take for example the topN selection for `white-breasted nuthatch`:

    eastern congo white-bellied water-snake: c:13 l:16
    chestnut-breasted negrofinch: c:13 l:5
    white-tailed crested-flycatcher: c:13 l:8
    chestnut white-bellied rat: c:13 l:3
    white-breasted cormorant: c:14 l:1
    white-breasted white-eye: c:14 l:1
    northern white-breasted hedgehog: c:15 l:9
    orange-breasted fruiteater: c:13 l:3
    white-breasted antbird: c:14 l:1
    chestnut-breasted whiteface: c:16 l:4
    white-breasted negrofinch: c:16 l:2
    white-breasted sunbird: c:14 l:1
    white-breasted tapaculo: c:14 l:0
    southern white-breasted hedgehog: c:16 l:9
    white-breasted thrasher: c:15 l:0
    white-breasted monarch: c:15 l:1
    white-breasted waterhen: c:15 l:0
    white-breasted hawk: c:15 l:4
    red-breasted nuthatch: c:17 l:2   << Highest coverage
    white-browed nuthatch: c:17 l:2   << Highest coverage

We can now run a fuzzy search for each heap item against the corresponding query using a partial ratio fuzz. 

    q:white-breasted nuthatch       r:eastern congo white-bellied water-snake       s:57 l:16, score: 69
    q:white-breasted nuthatch       r:white-tailed crested-flycatcher       s:52 l:8, score: 70
    q:white-breasted nuthatch       r:chestnut-breasted negrofinch  s:61 l:5, score: 86
    q:white-breasted nuthatch       r:chestnut white-bellied rat    s:65 l:3, score: 94
    q:white-breasted nuthatch       r:orange-breasted fruiteater    s:65 l:3, score: 94
    q:white-breasted nuthatch       r:white-breasted white-eye      s:70 l:1, score: 104
    q:white-breasted nuthatch       r:white-breasted sunbird        s:73 l:1, score: 108
    q:white-breasted nuthatch       r:white-breasted cormorant      s:70 l:1, score: 104
    q:white-breasted nuthatch       r:white-breasted antbird        s:77 l:1, score: 114
    q:white-breasted nuthatch       r:white-breasted tapaculo       s:78 l:0, score: 117
    q:white-breasted nuthatch       r:northern white-breasted hedgehog      s:74 l:9, score: 102
    q:white-breasted nuthatch       r:white-breasted hawk   s:84 l:4, score: 122
    q:white-breasted nuthatch       r:white-breasted monarch        s:82 l:1, score: 122
    q:white-breasted nuthatch       r:white-breasted thrasher       s:83 l:0, score: 124
    q:white-breasted nuthatch       r:white-breasted waterhen       s:78 l:0, score: 117
    q:white-breasted nuthatch       r:southern white-breasted hedgehog      s:74 l:9, score: 102
    q:white-breasted nuthatch       r:chestnut-breasted whiteface   s:62 l:4, score: 89
    q:white-breasted nuthatch       r:white-breasted negrofinch     s:70 l:2, score: 103
    q:white-breasted nuthatch       r:white-browed nuthatch s:81 l:2, score: 119
    q:white-breasted nuthatch       r:red-breasted nuthatch s:93 l:2, score: 137 << Highest score

The score (`s`) is the partial fuzz ratio, the length is the length difference with the query ( `l` ), and the score is calculated by `score = (FuzzScore * Scale) - l`. The `Scale` is a user parameter that determines how heavy the fuzz score weighs compared to the length. By default that is `2x`, so scores is twice as heavy as the length. In this case the best match is `red-breasted nuthatch`. While not the same it indeed seems a good choice. 

### Parameters explained 
We take five parameters of which one optional. For example:
`HeurFuzz example/test_query.txt .example/test_refs.txt 75 output.txt 1.5`
Those are, in order:

 - `query`, the file with all queries
 - `reference`, the file with all references
 - `FuzzCutOff`, the fuzzing score cut-off, 75% in the example. Note that this is just for the initial filter. The actual match will be selected based on the `score = (FuzzScore * Scale) - l` (so factoring in length as well).
 - `Scale`, how to weigh the fuzzing score (`FuzzScore`) compared to the length. A scale of `2` weighs fuzzing twice as heavy than the length difference between the query and reference. This is a float, so you can also let it weigh less, .e.g `0.9`. 
