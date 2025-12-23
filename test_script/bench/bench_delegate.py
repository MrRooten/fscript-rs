import asyncio

def yield_fn():
    for i in range(3_000_000):
        yield i

def delegate_test():
    yield from yield_fn()
    yield from yield_fn()
    yield from yield_fn()

def main():
    delegate_obj = delegate_test()
    for i in delegate_obj:
        pass

if __name__ == "__main__":
    main()