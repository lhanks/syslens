import { TestBed } from '@angular/core/testing';
import { StatusService } from './status.service';

describe('StatusService', () => {
  let service: StatusService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(StatusService);
  });

  afterEach(() => {
    service.clear();
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });

  describe('initial state', () => {
    it('should have "Ready" as initial status', () => {
      expect(service.currentStatus()).toBe('Ready');
    });

    it('should not be loading initially', () => {
      expect(service.isLoading()).toBeFalse();
    });

    it('should have zero operations initially', () => {
      expect(service.operationCount()).toBe(0);
    });
  });

  describe('startOperation', () => {
    it('should start tracking an operation', () => {
      service.startOperation('test-op', 'Loading data...');

      expect(service.isLoading()).toBeTrue();
      expect(service.operationCount()).toBe(1);
      expect(service.currentStatus()).toBe('Loading data...');
    });

    it('should track multiple operations', () => {
      service.startOperation('op1', 'Loading CPU...');
      service.startOperation('op2', 'Loading Memory...');

      expect(service.operationCount()).toBe(2);
      expect(service.isLoading()).toBeTrue();
    });

    it('should show most recent operation status', () => {
      service.startOperation('op1', 'First operation');

      // Small delay to ensure different timestamps
      jasmine.clock().install();
      jasmine.clock().tick(10);
      service.startOperation('op2', 'Second operation');
      jasmine.clock().uninstall();

      expect(service.currentStatus()).toBe('Second operation');
    });
  });

  describe('endOperation', () => {
    it('should remove a tracked operation', () => {
      service.startOperation('test-op', 'Loading...');
      service.endOperation('test-op');

      expect(service.isLoading()).toBeFalse();
      expect(service.operationCount()).toBe(0);
      expect(service.currentStatus()).toBe('Ready');
    });

    it('should not affect other operations', () => {
      service.startOperation('op1', 'First');
      service.startOperation('op2', 'Second');
      service.endOperation('op1');

      expect(service.operationCount()).toBe(1);
      expect(service.isLoading()).toBeTrue();
    });

    it('should handle ending non-existent operation', () => {
      service.endOperation('non-existent');

      expect(service.isLoading()).toBeFalse();
      expect(service.operationCount()).toBe(0);
    });
  });

  describe('updateOperation', () => {
    it('should update an existing operation message', () => {
      service.startOperation('test-op', 'Original message');
      service.updateOperation('test-op', 'Updated message');

      expect(service.currentStatus()).toBe('Updated message');
    });

    it('should not create new operation when updating non-existent', () => {
      service.updateOperation('non-existent', 'New message');

      expect(service.isLoading()).toBeFalse();
      expect(service.operationCount()).toBe(0);
    });
  });

  describe('clear', () => {
    it('should remove all operations', () => {
      service.startOperation('op1', 'First');
      service.startOperation('op2', 'Second');
      service.startOperation('op3', 'Third');

      service.clear();

      expect(service.isLoading()).toBeFalse();
      expect(service.operationCount()).toBe(0);
      expect(service.currentStatus()).toBe('Ready');
    });
  });
});
