
from collections import defaultdict
from heapq import heapify, heappush, heappushpop, nlargest
import numpy as np 

class MaxHeap():
    def __init__(self, top_n):
        self.h = []
        self.length = top_n
        heapify(self.h)
        
    def add(self, element):
        if len(self.h) < self.length:
            heappush(self.h, element)
        else:
            heappushpop(self.h, element)
            
    def getTop(self):
        return nlargest(self.length, self.h)

class Node(object):
    def __init__(self, coverage, length_difference):
        self.cov = coverage 
        self.length_difference = length_difference

    def __repr__(self):
        return f'Cov: {self.cov} & L: {self.length_difference}'

    def __lt__(self, other):
        if self.cov != other.cov:
            return self.cov < other.cov
        else: 
            return self.length_difference < other.length_difference
        
def generate_bigrams(lst):
    return [(lst[i], lst[i + 1]) for i in range(len(lst) - 1)]
    
def index_queries(queries, top_n = 2):
    heaps = dict()
    index = defaultdict(lambda: defaultdict(int)) 
    for (i, query) in enumerate(queries):
        heaps[i] = MaxHeap(top_n)
        for bigram in generate_bigrams(query):
            index[bigram][i] += 1
    return index, heaps 

def query_index(index, heaps, ref):
    for bigram in generate_bigrams(ref):
        entry = index.get(bigram)
        if entry:
            for (seq_id, count) in entry.items():
                
                                    

query = [[1,1,2], [1,2,3]]
refs = [[1,1,2,3], [1,1,3], [1,2,3]]        
index, heaps = index_queries(query)

            
m = MaxHeap(2)
for item in [Node(20, 2), Node(20, 5), Node(20, 3), Node(100, 1), Node(100, 2)]:
    m.add(item)

for x in m.h:
    print(x)


def query_it(i, query):
    for bigram in generate_bigrams(query):
        entry = i.get(bigram)
        if entry:
            for (k,v) in entry.items():
                print(bigram, " ; ", k, "-> ", v)  
    
    
# query = [1,1,3,4]
# data = [[1, 1, 2], [1, 1, 3, 4], [1, 1]]
# i = index_it(data)
# query_it(i, query)
# Key 
# AA  (10: 1, 30: 4, 40: 100)
 