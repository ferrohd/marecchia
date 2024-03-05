export class LRUMap<T> {
    private maxCapacity: number;
    private map: Map<string, T>;

    constructor(maxCapacity: number = 10) {
        this.maxCapacity = maxCapacity;
        this.map = new Map();
    }

    // Adds an item to the set or updates its position if it already exists
    set(key: string, value: T): void {
        if (this.map.has(key)) {
            // Item exists; remove and reinsert to update its "recently used" status
            this.map.delete(key);
        } else if (this.map.size >= this.maxCapacity) {
            // Evict the least recently used (LRU) item, which is the first one in the Map
            const lruKey = this.map.keys().next().value;
            this.map.delete(lruKey);
        }
        this.map.set(key, value); // The value 'true' is arbitrary since we're emulating a Set
    }

    // Removes an item from the set
    delete(key: string): boolean {
        return this.map.delete(key);
    }

    // Checks if the item exists in the set and moves it to the most recently used position
    get(key: string): T | null {
        const value = this.map.get(key);
        if (!value) return null;
        this.map.delete(key);
        this.map.set(key, value);
        return value;
    }

    // Clears all items from the set
    clear(): void {
        this.map.clear();
    }

    get size(): number {
        return this.map.size;
    }

    // An iterator to allow easy iteration over the set, in the order of most recently used to least
    [Symbol.iterator](): Iterator<T> {
        return this.map.values();
    }
}
