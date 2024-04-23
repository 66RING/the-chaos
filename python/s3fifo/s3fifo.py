'''
S3FIFO implement

Insert: if not in ghost, insert into small, else insert into main
Evict(small): if not visited, insert into ghost, else insert into main
Evict(main): if visited, insert back, else evict
'''

from collections import deque
from dataclasses import dataclass
from typing import Hashable
from enum import Enum, auto

class Location(Enum):
    GHOST = auto()
    SMALL = auto()
    MAIN = auto()

@dataclass
class S3FIFOItem:
    key: Hashable
    value: any
    visited_cnt: int = 0
    location: Location = Location.SMALL

class S3FIFO:
    def __init__(self, main_size, small_size):
        # an N chance FIFO
        self.main_queue = deque()
        # an 01 chance FIFO
        self.small_queue = deque()
        # an FIFO
        self.ghost_queue = deque()

        self.item_table = {}

        self.cache_size = main_size + small_size
        self.small_size = small_size
        self.main_size = main_size
        self.ghost_size = small_size

    def put(self, key, value):
        if self.item_table.get(key) is None:
            # if not exist, create one
            item = S3FIFOItem(key, value)
            self.item_table[key] = item
            self.insert_small(item)
        else:
            item = self.item_table[key]
            if item.location == Location.GHOST:
                # if in ghost, insert into main
                self.ghost_queue.remove(item)
                print("remove from ghost",item)
                self.insert_main(item)
            else:
                # else insert into main
                item.visited_cnt += 1

    def get(self, key):
        if self.item_table.get(key) is None:
            return None
        else:
            item = self.item_table[key]
            item.visited_cnt += 1
            return item.value

    def has(self, key):
        return self.item_table.get(key) is not None

    def insert_main(self, item):
        '''
        1. renew visited
        2. insert
        '''
        item.location = Location.MAIN
        item.visited_cnt = 0
        self.try_evict_main()
        self.main_queue.appendleft(item)

    def try_evict_main(self):
        if len(self.main_queue) < self.main_size:
            return

        # evict one item that cnt is 0
        while len(self.main_queue) > 0:
            # pop right
            item = self.main_queue.pop()
            if item.visited_cnt > 0:
                item.visited_cnt -= 1
                self.main_queue.appendleft(item)
            else:
                # evict item that visited_cnt is 0
                item.value = None
                del self.item_table[item.key]
                return

        # unreachable
        raise Exception('unreachable code in main')

    def insert_small(self, item):
        item.location = Location.SMALL
        item.visited_cnt = 0
        self.try_evict_small()
        self.small_queue.appendleft(item)

    def try_evict_small(self):
        '''
        Evict(small): if not visited, insert into ghost, else insert into main
        '''
        if len(self.small_queue) < self.small_size:
            return

        item = self.small_queue.pop()
        if item.visited_cnt == 0:
            # insert into ghost
            self.insert_ghost(item)
        else:
            # insert into main
            self.insert_main(item)

    def insert_ghost(self, item):
        item.location = Location.GHOST
        item.visited_cnt = 0
        self.try_evict_ghost()
        self.ghost_queue.appendleft(item)

    def try_evict_ghost(self):
        if len(self.ghost_queue) < self.ghost_size:
            return
        # simply pop right one and remove from item_table
        item = self.ghost_queue.pop()
        item.value = None
        del self.item_table[item.key]

    def print(self):
        print('main:\n', self.main_queue)
        print('small:\n', self.small_queue)
        print('ghost:\n', self.ghost_queue)


def test_s3fifo():
    items = [1, 2, 1, 3, 4, 5, 5, 1, 6, 1, 4, 1, 7, 1]
    '''
    main : 5(1) 4(0) 1(3)
    small: 7(0) 6(0)
    ghost: 3(0)
    '''
    s3fifo = S3FIFO(10, 2)
    for item in items:
        s3fifo.put(item, item)
        s3fifo.print()
        input()


test_s3fifo()


