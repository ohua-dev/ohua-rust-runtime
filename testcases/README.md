# Testcases

This folder contains the test suite that checks and verifies the correct function and integration of the compiler plugin itself as well as the operators that are part of Ohua.
These tests are run as part of the CI process after every push to `master` but if you want to run them yourself, just execute

```
./run_tests.sh
```

The script assumes every folder contains a valid Cargo project and tries to execute it. To exclude a test from execution, place a file named `.skipfile` in the test case folder.

## Function tests

| Test Case                                                 | What is tested?                                                                                                                                           |
| ---------                                                 | ---------------                                                                                                                                           |
| [`ohua_macro`](ohua_macro/)                               | general functionality; Does the compiler plugin hook in correctly into the build process? Are the files generated correctly to form a working executable? |
| [`argument_clone`](argument_clone/)                       | Are `clone`s places correctly in the algorithm where necessary to duplicate an argument to a function?                                                    |
| [`custom_types`](custom_types/)                           | Do custom defined types (i.e., structs and enums) work as expected? Can they be used in an algorithm? Are imports placed correctly?                       |
| [`custom_types_envarcs`](custom_types_envarcs/)           | Can custom types be handed over via environment arcs (i.e., as arguments to the algorithm)?                                                               |
| [`lambdas`](lambdas/)                                     | Do lambda functions work properly?                                                                                                                        |
| [`mainargs`](mainargs/)                                   | Do arguments to the algorithm (read: envarcs) work in general? Are the arguments moved to the correct threads?                                            |
| [`mainargs_clone`](mainargs_clone/)                       | Is data from environment arcs cloned correctly if necessary?                                                                                              |
| [`mainargs_reuse_across_ops`](mainargs_reuse_across_ops/) | Can we put (cloned) envarc data into different threads or does that pose a problem to thread safety?                                                      |

## Operator tests

| Test Case                                 | What is tested?                                                     |
| ---------                                 | ---------------                                                     |
| [`smap_test`](smap_test/)                 | general `smap` functionality                                        |
| [`smap_with_lambdas`](smap_with_lambdas/) | using lambda functions inside `smap`                                |
| [`smap_with_envarcs`](smap_with_envarcs/) | using environment values in `smap`                                  |
| [`if_test`](if_test/)                     | general `if` functionality                                          |
| [`if_with_lambdas`](if_with_lambdas/)     | using lambda functions inside `if`                                  |
| [`if_with_envarcs`](if_with_envarcs/)     | using environment values in `if` (either ctrl input or in a branch) |
| [`if_in_if`](if_in_if/)                   | are nested `if`s working?                                           |
| [`smap_in_if`](smap_in_if/)               | `smap` nested in `if`                                               |
