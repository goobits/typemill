// Deeply nested structures for testing

interface Level1 {
  level2: {
    level3: {
      level4: {
        level5: {
          level6: {
            level7: {
              level8: {
                level9: {
                  level10: {
                    deepValue: string;
                    deepMethod(): void;
                  };
                };
              };
            };
          };
        };
      };
    };
  };
}

class DeeplyNestedClass {
  static readonly DEEP = {
    nested: {
      structure: {
        with: {
          many: {
            levels: {
              of: {
                nesting: {
                  value: 'deep',
                  method: function() {
                    return function() {
                      return function() {
                        return function() {
                          return 'very deep';
                        };
                      };
                    };
                  }
                }
              }
            }
          }
        }
      }
    }
  };

  constructor() {
    // Deeply nested anonymous functions
    const nested = () => {
      return () => {
        return () => {
          return () => {
            return () => {
              return () => {
                return () => {
                  return 'nested arrow functions';
                };
              };
            };
          };
        };
      };
    };
  }
}

// Deeply nested object literal
const deepObject = {
  a: {
    b: {
      c: {
        d: {
          e: {
            f: {
              g: {
                h: {
                  i: {
                    j: {
                      k: {
                        l: {
                          m: 'deep value'
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
};

// Deeply nested type
type DeepType<T> = {
  value: T;
  next: DeepType<DeepType<DeepType<DeepType<DeepType<T>>>>>;
};

// Export for testing
export { Level1, DeeplyNestedClass, deepObject, DeepType };