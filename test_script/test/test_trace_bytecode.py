import dis
import sys
from types import FrameType
from typing import Any
from collections import Counter
counter = Counter()

def trace(frame: FrameType, event: str, arg: Any):
    frame.f_trace_opcodes = True
    if event == 'opcode':
        code = frame.f_code.co_code
        lasti = frame.f_lasti
        opcode = code[lasti]
        opname = dis.opname[opcode]
        counter[opname] += 1
    return trace


sys.settrace(trace)
def fib(n):
    if n == 1 or n == 2:
        return 1
def abc():
    ## The test function must placed in here if not it will not be traced
    for i in range(1800):
        if i == 0:
            pass

abc()

sys.settrace(None)
print(counter)
print("Total opcodes executed:", sum(counter.values()))