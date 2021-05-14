import heapq;
import math;

# edge: (n1, n2, discriminator, length, iidx)

def kreek(nodes, edges):
	""" Duplicates edges in the graph to make it eulirian """
	ne = {}
	for n in nodes: ne[n] = []
	def dupedge(e):
		e = (e[0], e[1], e[2], e[3], e[4]+1)
		ne[e[0]].append(e)
		ne[e[1]].append(e)
	for e in edges: dupedge((e[0], e[1], e[2], e[3], -1))
	# dead ends begoneth!
	for (n, es) in ne.items():
		if len(es) == 1: dupedge(es[0])
	# eliminate the odds
	while True:
		(n, es) = next(((n, es) for (n, es) in ne.items() if len(es) % 2 == 1), (None, None))
		if n is None: break
		e1s = sorted([e for e in es if len([e2 for e2 in es if e2[0] == e[0] and e2[1] == e[1] and e2[2] == e[2]]) == 1], key = lambda e: e[3])
		dupedge(e1s[0])
	# what do we got?
	return ne

def other(n, e):
	return e[1] if e[0] == n else e[0] 

def remedge(ne, e):
	ne[e[0]].remove(e)
	ne[e[1]].remove(e)

def dijkstra_on_a_cycle(n0, ne):
	""" Finds the shortest (non-trivial) [undirected] cycle on n0 """
	ne = ne.copy()
	dist = {}
	pafs = {}
	dist[n0] = math.inf
	pafs[n0] = []
	q = [(dist[n0], n0)]
	heapq.heapify(q)
	while len(q) > 0:
		(_, n) = heapq.heappop(q)
		if n is None: continue
		nd = dist[n]
		if n == n0:
			if nd < math.inf: return pafs[n]
			else: nd = 0
		for e in ne[n]:
			path = pafs[n]
			if e in path: continue
			v = other(n, e)
			vd = nd + e[3]
			if v not in dist or vd < dist[v]:
				path = path.copy()
				path.append(e)
				dist[v] = vd
				pafs[v] = path
				for i in range(len(q)):
					if q[i][1] == v: q[i] = (q[i][0], None)
				heapq.heappush(q, (vd, v))
	return None

# 1. Take a starting vertex
# 2. Find a shortest path to itself
# 3. Repeat with other starting vertices
# 4. Take a partial cycle
# 5. Find closest vertex to the starting vertex with remaining edges
# 6. Find shortest path to itself
# 7. Inject into the cycle
# 8. Repeat with next partial cycle
# 9. Repeat until all edges are traversed

def bl33p(ne, sns):
	""" Find list of paths over eulirian graph that together all edges starting from specified positions """
	ne = ne.copy()
	cycles = {}
	for n in sns: cycles[n] = []
	i = 0
	while sum([len(es) for es in ne.values()]) > 0:
		n = sns[i]
		cycle = cycles[n]
		v = n
		y = 0
		if len(cycle) > 0 and len(ne[v]) == 0:
			v1 = v
			for k in range(len(cycle)):
				v2 = other(v1, cycle[k])
				if len(ne[v2]) > 0:
					v = v2
					y = k+1
					break
				else: v1 = v2
			if v == n: v = None
		if v is not None:
			inj = dijkstra_on_a_cycle(v, ne)
			cycle[y:y] = inj
			for e in inj: remedge(ne, e)
		i = (i+1)%len(sns)
	return cycles

E = [(0, 1, None, 1), (1, 2, "a", 1), (1, 2, "b", 3), (1, 2, "c", 5), (2, 0, None, 5)]
G = kreek([0, 1, 2, 3], E)
print(G)
# print(dijkstra_on_a_cycle(0, G))
print(bl33p(G, [0, 2]))