import re
import math

register = r'(?:\b[a-zA-Z][a-zA-Z0-9]*\b)'
number = r'(?:\d+\.?\d*)'
regOrNum = r'(?:'+register+r'|'+number+')'

# Assigning instruction
def asgnI(name, nargs):
    return name+r'\s+('+register+')\s+'+'\s+'.join('('+regOrNum+')' for _ in range(nargs))

unaryInstrs = [asgnI(name, 1) for name in [
        'l',
        'lb',
        ]]
binaryInstrs = [asgnI(name, 2) for name in [
        'add',
        'div',
        'mul',
        'sub',
        ]]
instrs = unaryInstrs + binaryInstrs
patterns = [re.compile(p) for p in instrs]+[
        re.compile(r's\s*
        ]

class Lifetime:
    def __init__(self, n, s, e):
        self.n = n
        self.s = s
        self.e = e


def profile(lines):
    lifetimes = {}
    living = {}
    for (i, line) in enumerate(lines):
        for p in patterns:
            m = p.match(line)
            if m:
                print(line)
                groups = m.groups()
                r = groups[1]
                for j in range(2, len(groups)):
                    # living[groups[j]].e = i
                    pass
                if r in living:
                    lt = living[r]
                    lt.e = i
                    lifetimes += [lt]
                lt = lifetimes.get(r, [])
                lt += [Lifetime(r, i, i)]

                # print(' '.join(m.groups()))
                # for g in m.groups():
                    # print(g)
    return lifetimes

def getLines(srcpath):
    f = open(srcpath)
    lines = f.read().splitlines()
    f.close()
    return lines


if __name__ == '__main__':
    import sys
    srcpath = 'test.mips'
    lines = getLines(sys.argv[1])
    lifetimes = profile(lines)

    for (k, v) in lifetimes.items():
        print(k, v)

    # iw = math.ceil(math.log10(len(lifetimes)))
    # lw = math.ceil(math.log10(len(lines)))
    # fmt = f's_{{:0{iw}}} ({{:{lw}}},{{:{lw}}})'
    # print('s_i'+' '*iw+'('+' '*(lw-1)+'s,'+' '*(lw-1)+'e) [s: start line, e: end line]')
    # for lt in lifetimes:
        # print(lt.i)
        # print(fmt.format(lt.i, lt.s, lt.e))
