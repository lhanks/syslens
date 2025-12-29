import { BytesPipe } from './bytes.pipe';

describe('BytesPipe', () => {
  let pipe: BytesPipe;

  beforeEach(() => {
    pipe = new BytesPipe();
  });

  it('should create an instance', () => {
    expect(pipe).toBeTruthy();
  });

  describe('auto-unit selection', () => {
    it('should format bytes', () => {
      expect(pipe.transform(0)).toBe('0 B');
      expect(pipe.transform(500)).toBe('500 B');
      expect(pipe.transform(1023)).toBe('1023 B');
    });

    it('should format kilobytes', () => {
      expect(pipe.transform(1024)).toBe('1 KB');
      expect(pipe.transform(1536)).toBe('1.5 KB');
      expect(pipe.transform(10240)).toBe('10 KB');
    });

    it('should format megabytes', () => {
      expect(pipe.transform(1048576)).toBe('1 MB');
      expect(pipe.transform(1572864)).toBe('1.5 MB');
      expect(pipe.transform(104857600)).toBe('100 MB');
    });

    it('should format gigabytes', () => {
      expect(pipe.transform(1073741824)).toBe('1 GB');
      expect(pipe.transform(8589934592)).toBe('8 GB');
      expect(pipe.transform(17179869184)).toBe('16 GB');
    });

    it('should format terabytes', () => {
      expect(pipe.transform(1099511627776)).toBe('1 TB');
      expect(pipe.transform(2199023255552)).toBe('2 TB');
    });
  });

  describe('target unit conversion', () => {
    it('should convert to specified unit', () => {
      expect(pipe.transform(1073741824, 'MB')).toBe('1024 MB');
      expect(pipe.transform(1073741824, 'GB')).toBe('1 GB');
      expect(pipe.transform(1048576, 'KB')).toBe('1024 KB');
    });

    it('should handle case-insensitive unit', () => {
      expect(pipe.transform(1073741824, 'gb')).toBe('1 GB');
      expect(pipe.transform(1048576, 'mb')).toBe('1 MB');
    });

    it('should fall back to auto-select for invalid unit', () => {
      expect(pipe.transform(1024, 'INVALID')).toBe('1 KB');
    });
  });

  describe('precision', () => {
    it('should use default precision of 1', () => {
      expect(pipe.transform(1536)).toBe('1.5 KB');
    });

    it('should use custom precision', () => {
      expect(pipe.transform(1536, '', 2)).toBe('1.5 KB');
      expect(pipe.transform(1234567, '', 2)).toBe('1.18 MB');
      expect(pipe.transform(1234567, '', 0)).toBe('1 MB');
    });

    it('should remove trailing zeros', () => {
      expect(pipe.transform(1024, '', 3)).toBe('1 KB');
      expect(pipe.transform(1536, '', 3)).toBe('1.5 KB');
    });
  });

  describe('edge cases', () => {
    it('should handle null', () => {
      expect(pipe.transform(null)).toBe('0 B');
    });

    it('should handle undefined', () => {
      expect(pipe.transform(undefined)).toBe('0 B');
    });

    it('should handle NaN', () => {
      expect(pipe.transform(NaN)).toBe('0 B');
    });

    it('should handle negative numbers', () => {
      // Pipe handles absolute value for unit calculation
      const result = pipe.transform(-1024);
      expect(result).toContain('KB');
    });

    it('should handle very large numbers', () => {
      // 1 PB
      expect(pipe.transform(1125899906842624)).toBe('1 PB');
    });
  });
});
