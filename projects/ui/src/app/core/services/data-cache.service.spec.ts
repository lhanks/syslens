import { TestBed } from '@angular/core/testing';
import { DataCacheService, CacheKeys } from './data-cache.service';

describe('DataCacheService', () => {
  let service: DataCacheService;

  beforeEach(() => {
    // Clear localStorage before each test
    localStorage.clear();

    TestBed.configureTestingModule({});
    service = TestBed.inject(DataCacheService);
  });

  afterEach(() => {
    service.clearAll();
    localStorage.clear();
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });

  describe('save and load', () => {
    it('should save and load simple data', () => {
      const testData = { name: 'Test CPU', cores: 8 };
      service.save(CacheKeys.CPU_INFO, testData);

      const loaded = service.load<typeof testData>(CacheKeys.CPU_INFO);
      expect(loaded).toEqual(testData);
    });

    it('should save and load complex data', () => {
      const testData = {
        adapters: [
          { name: 'Ethernet', ip: '192.168.1.1' },
          { name: 'WiFi', ip: '192.168.1.2' }
        ],
        count: 2
      };
      service.save(CacheKeys.NETWORK_ADAPTERS, testData);

      const loaded = service.load<typeof testData>(CacheKeys.NETWORK_ADAPTERS);
      expect(loaded).toEqual(testData);
    });

    it('should return null for non-existent key', () => {
      const loaded = service.load(CacheKeys.GPU_INFO);
      expect(loaded).toBeNull();
    });

    it('should persist data to localStorage', () => {
      const testData = { model: 'RTX 4090' };
      service.save(CacheKeys.GPU_INFO, testData);

      // Create a new service instance to test localStorage persistence
      const newService = new DataCacheService();
      const loaded = newService.load<typeof testData>(CacheKeys.GPU_INFO);
      expect(loaded).toEqual(testData);
    });
  });

  describe('has', () => {
    it('should return true for existing entry', () => {
      service.save(CacheKeys.CPU_INFO, { name: 'Test' });
      expect(service.has(CacheKeys.CPU_INFO)).toBeTrue();
    });

    it('should return false for non-existent entry', () => {
      expect(service.has(CacheKeys.GPU_INFO)).toBeFalse();
    });

    it('should check expiration with maxAgeMs', () => {
      service.save(CacheKeys.CPU_INFO, { name: 'Test' });

      // Should exist with a large maxAge
      expect(service.has(CacheKeys.CPU_INFO, 60000)).toBeTrue();

      // Should not exist with a tiny maxAge (data is already "old")
      expect(service.has(CacheKeys.CPU_INFO, 0)).toBeFalse();
    });
  });

  describe('getTimestamp', () => {
    it('should return timestamp for cached entry', () => {
      const beforeSave = Date.now();
      service.save(CacheKeys.CPU_INFO, { name: 'Test' });
      const afterSave = Date.now();

      const timestamp = service.getTimestamp(CacheKeys.CPU_INFO);

      expect(timestamp).not.toBeNull();
      expect(timestamp).toBeGreaterThanOrEqual(beforeSave);
      expect(timestamp).toBeLessThanOrEqual(afterSave);
    });

    it('should return null for non-existent entry', () => {
      expect(service.getTimestamp(CacheKeys.GPU_INFO)).toBeNull();
    });
  });

  describe('clear', () => {
    it('should clear a specific entry', () => {
      service.save(CacheKeys.CPU_INFO, { name: 'CPU' });
      service.save(CacheKeys.GPU_INFO, { name: 'GPU' });

      service.clear(CacheKeys.CPU_INFO);

      expect(service.has(CacheKeys.CPU_INFO)).toBeFalse();
      expect(service.has(CacheKeys.GPU_INFO)).toBeTrue();
    });
  });

  describe('clearAll', () => {
    it('should clear all cached entries', () => {
      service.save(CacheKeys.CPU_INFO, { name: 'CPU' });
      service.save(CacheKeys.GPU_INFO, { name: 'GPU' });
      service.save(CacheKeys.MEMORY_INFO, { size: 32 });

      service.clearAll();

      expect(service.has(CacheKeys.CPU_INFO)).toBeFalse();
      expect(service.has(CacheKeys.GPU_INFO)).toBeFalse();
      expect(service.has(CacheKeys.MEMORY_INFO)).toBeFalse();
    });
  });

  describe('getStats', () => {
    it('should return correct entry count', () => {
      const initialStats = service.getStats();
      const initialCount = initialStats.entries;

      service.save(CacheKeys.CPU_INFO, { name: 'CPU' });
      service.save(CacheKeys.GPU_INFO, { name: 'GPU' });

      const stats = service.getStats();

      expect(stats.entries).toBe(initialCount + 2);
      expect(stats.totalSize).toBeGreaterThan(0);
      expect(stats.lastUpdated).not.toBeNull();
    });

    it('should increase entry count when saving', () => {
      const before = service.getStats().entries;
      service.save(CacheKeys.CPU_INFO, { name: 'Test' });
      const after = service.getStats().entries;

      expect(after).toBe(before + 1);
    });
  });

  describe('loadAll', () => {
    it('should load all cached entries', () => {
      service.save(CacheKeys.CPU_INFO, { name: 'CPU' });
      service.save(CacheKeys.GPU_INFO, { name: 'GPU' });

      const all = service.loadAll();

      expect(all.size).toBe(2);
      expect(all.get(CacheKeys.CPU_INFO)).toEqual({ name: 'CPU' });
      expect(all.get(CacheKeys.GPU_INFO)).toEqual({ name: 'GPU' });
    });

    it('should return empty map when no cache', () => {
      const all = service.loadAll();
      expect(all.size).toBe(0);
    });
  });

  describe('memory cache', () => {
    it('should serve from memory on second load', () => {
      const testData = { name: 'Test' };
      service.save(CacheKeys.CPU_INFO, testData);

      // First load populates memory cache
      service.load(CacheKeys.CPU_INFO);

      // Clear localStorage but memory cache should still work
      localStorage.clear();

      // Should still get data from memory cache
      const loaded = service.load(CacheKeys.CPU_INFO);
      expect(loaded).toEqual(testData);
    });
  });
});
