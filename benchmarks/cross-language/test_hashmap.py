import time

def hash_map_ops(n):
    map = {}
    i = 0
    while i < n:
        map[i] = i * 2
        i += 1
    sum = 0
    i = 0
    while i < n:
        sum += map[i]
        i += 1
    return sum

# Medium test
print("Testing with n=1000...")
start = time.perf_counter()
result = hash_map_ops(1000)
elapsed = (time.perf_counter() - start) * 1000
print(f"Result: {result}")
print(f"Time: {elapsed}ms")

# Large test
print("\nTesting with n=100000...")
start = time.perf_counter()
result = hash_map_ops(100000)
elapsed = (time.perf_counter() - start) * 1000
print(f"Result: {result}")
print(f"Time: {elapsed}ms")
