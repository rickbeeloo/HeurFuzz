
from collections import defaultdict
from heapq import heapify, heappush, heappushpop, nlargest
import numpy as np 

      
def generate_bigrams(lst):
    return [(lst[i], lst[i + 1]) for i in range(len(lst) - 1)]
    
def index_queries(queries):
    index = defaultdict(lambda: defaultdict(int)) 
    for (i, query) in enumerate(queries):
        for bigram in generate_bigrams(query):
            index[bigram][i] += 1
    return index

def query_index(index, refs, mat):
    for (j, ref) in enumerate(refs):
        for bigram in generate_bigrams(ref):
            entry = index.get(bigram)
            if entry:
                for (seq_id, count) in entry.items():
                    mat[j][seq_id] += count 
    return mat                
                                    

query = [[1,1,2], [1,2,3]]
refs = [[1,1,2,3], [1,1,3], [1,2,3]]  
m = np.zeros((len(refs), len(query)))      
index = index_queries(query)
mat = query_index(index, refs, m)


            
