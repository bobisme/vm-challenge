#!/usr/bin/env python

from dataclasses import dataclass
import heapq
from typing import Callable, Deque, Generator, NamedTuple


class Pos(NamedTuple):
    i: int
    j: int

    def up(self):
        return Pos(self.i - 1, self.j)

    def right(self):
        return Pos(self.i, self.j + 1)

    def down(self):
        return Pos(self.i + 1, self.j)

    def left(self):
        return Pos(self.i, self.j - 1)


START = Pos(3, 0)
END = Pos(0, 3)

Op = Callable[[int, int], int]


mul = int.__mul__
add = int.__add__
sub = int.__sub__


TARGET = 30

GRID = [
    [mul, 8, sub, 1],
    [4, mul, 11, mul],
    [add, 4, sub, 18],
    [22, sub, 9, mul],
]


def at(pos: Pos):
    return GRID[pos.i][pos.j]


def neighbors(pos: Pos) -> Generator[Pos]:
    for p in (pos.up(), pos.right(), pos.down(), pos.left()):
        if 0 <= p.i < len(GRID) and 0 <= p.j < len(GRID[0]):
            yield p


def minimal_paths() -> Generator[list[Pos]]:
    class Node(NamedTuple):
        pos: Pos
        prev_path: list[Pos]

    visited: set[Pos] = set()
    q = Deque()
    q.append(Node(START, []))

    while len(q) > 0:
        node = q.popleft()
        visited.add(node.pos)
        path = node.prev_path + [node.pos]
        if node.pos == END:
            yield path
            continue
        for n_pos in neighbors(node.pos):
            if n_pos in visited:
                continue
            q.append(Node(n_pos, path))


def eval(path: list[Pos]) -> int:
    head = path[0]
    out = GRID[head.i][head.j]
    for i in range(1, len(path), 2):
        op = GRID[path[i].i][path[i].j]
        num = GRID[path[i + 1].i][path[i + 1].j]
        out = op(out, num)
    return out


def dir(from_: Pos, to: Pos) -> str:
    if from_.i < to.i:
        return "south"
    elif from_.i > to.i:
        return "north"
    elif from_.j < to.j:
        return "east"
    else:
        return "west"


def print_path(path: list[Pos]):
    for i in range(len(path) - 1):
        print(dir(path[i], path[i + 1]))


def search():
    @dataclass
    class Node:
        pos: Pos
        prev_path: list[Pos]
        val: int

        def __lt__(self, other: "Node") -> bool:
            return self.h() < other.h()

        @property
        def diff(self) -> int:
            return abs(self.val - TARGET)

        @property
        def dist(self) -> int:
            return abs(END.i - self.pos.i) + abs(END.j - self.pos.j)

        def h(self) -> tuple[int, int, int]:
            return (len(self.prev_path), self.dist, self.diff)

    # q = [Node(Pos(1, 0), [START, Pos(2, 0)], 26)]
    q = [Node(START, [], at(START))]

    while len(q) > 0:
        node = heapq.heappop(q)
        path = node.prev_path + [node.pos]
        if node.pos == END and node.diff == 0:
            return path
        for op_pos in neighbors(node.pos):
            op = at(op_pos)
            op_path = path + [op_pos]
            for val_pos in neighbors(op_pos):
                # looks like we can backtrack, but not to the start
                if val_pos == START:
                    continue
                val = op(node.val, at(val_pos))
                if val < 0:  # negative values cause the orb to shatter
                    continue
                n = Node(val_pos, op_path, val)
                heapq.heappush(q, n)


def main():
    for path in minimal_paths():
        if eval(path) == TARGET:
            print("FOUND PATH", path)
            return

    path = search()
    if path:
        print("PATH FOUND:\n")
        print_path(path)


if __name__ == "__main__":
    main()
