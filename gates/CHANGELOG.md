# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.1 (2026-02-15)

<csr-id-46bee42d4f28d185b38446c0d950c2579c422f43/>
<csr-id-c987a225570c2afae480800327d0072ab4b4e4ad/>
<csr-id-bea47e8ee97b86a3120b8097d0fdbe6bc9fce133/>
<csr-id-dcf9154b305c79728dd2a9f61e4440b5a15756ea/>

### Chore

 - <csr-id-46bee42d4f28d185b38446c0d950c2579c422f43/> update dependencies and align workspace configurations
   - Updated various dependencies in Cargo.toml files across multiple crates to their latest versions for improved functionality and compatibility.
   - Changed several dependencies to use workspace references for consistency and to reduce duplication.
   - Notable updates include polars to version 0.53.0, faer to version 0.24, and ndarray-linalg to version 0.18.1.
   - Adjusted dev-dependencies to utilize workspace settings for better management.
 - <csr-id-c987a225570c2afae480800327d0072ab4b4e4ad/> clean up unused imports and variables
   - Remove unused imports in clustering and gating modules
   - Fix unreachable code warning in DBSCAN
   - Remove unused mut keywords
   - Clean up warnings for better code quality

### Chore

 - <csr-id-089feff624625a5ddf0b1da570e4f60b6fedf09b/> update changelogs prior to release

### Documentation

<csr-id-6f6d0f59369453e3f0018b37f1377b204b023223/>
<csr-id-f9eef00689d5c1dbda8bce37ca0d399afae19d46/>

 - <csr-id-69d65c959a392f16431cc98beae9c361ccfed10a/> add implementation status document
   - Document completed features and known limitations
- Note performance targets and achievements
- List future enhancements
- Update testing status
- Add README for flow-utils crate with usage examples
- Add CRATE_RESEARCH.md documenting crate evaluation and decisions
- Add RESEARCH_NOTES.md for automated gating algorithms and decisions
- Document performance vs accuracy tradeoffs
- Note known limitations and future work
- Document scatter gating methods and usage
- Document doublet detection methods and usage
- Document preprocessing pipeline
- Include algorithm details and performance notes
- Note known limitations (clustering API, multi-population)

### New Features

<csr-id-0e1ee96078a18b06ce5c0c8776df9892d7861ea8/>
<csr-id-5996edc676f6a606fcd48e2ffc8ed3131f08ce0b/>
<csr-id-547c2ae09f0f263314de70750b8c8e01b4fd4661/>
<csr-id-c0ba8e72f6866bda5d9eec40a6f089ccc7c35107/>
<csr-id-340977390c10a31fdf7694ac9325147f406c5b72/>
<csr-id-6a65bd7077b2a12670c3766248b08447e92ea8b5/>
<csr-id-43a00f6f0e4043d9b973eb8c9ae2c18ff64b780d/>
<csr-id-c89944be9c68a1f688dfb5ee333c7562b28f90b1/>
<csr-id-7b65fbcc9119762ee4cf64cf129c017ece95ff30/>
<csr-id-c998c06382ec30a870452083b7366a74ced5830e/>
<csr-id-6762e5f0d484be7e8d45363205793a50e46b0eb3/>

 - <csr-id-42b46207448be5ca137b0b1067ddaa1222b50ccb/> add hierarchy support and gating improvements
   - Extend hierarchy module
- Minor updates to scatter, filtering, gatingml
- Triple total events: 20k -> 60k (most scenarios), 30k -> 90k (with_debris)
- Reduce FSC mean by 35% for single-population and with_doublets (50000 -> 32500)
- Narrow all distributions by ~40% for more concentration (except debris)
- Debris population keeps original wider distribution
- All plots regenerated with new parameters
- Add generate_with_debris function to test_helpers.rs
- 15% debris population near origin (low FSC/SSC)
- All 5 scenarios now use Gaussian distributions
- Tests should compile and pass
- Replace all uniform random generation with Normal distributions
- Use rand_distr::Normal for realistic flow cytometry data patterns
- Proper correlations between FSC-A/FSC-H and SSC-A/SSC-H
- All scenarios now generate Poisson/Gaussian-like distributions
- Fixes compilation errors and improves data realism
- Replace uniform random blocks with Gaussian distributions using rand_distr
- Add proper correlations between FSC-A/FSC-H and SSC-A/SSC-H
- Increase event density (20k events for visualization)
- Add WithDebris scenario: 15% debris population near origin (low FSC/SSC)
- Debris population tests automated gating's ability to ignore low events
- All scenarios now use Poisson/Gaussian-like distributions for realism
- Add peak_bias_negative for left-side biasing of negative peaks
- Keep peak_bias for right-side biasing of positive peaks
- Both configurable via CLI flags
- Implement extract_negative_event_autofluorescence function
- Use peak detection to find negative peak (left/low peak)
- Calculate AF medians from negative events per endmember
- Support threshold-based fallback method
- Store negative event AF separately from universal AF
- Implement autofluorescence_mode selection (universal, negative-events, hybrid)
- Weighted combination: α * universal + (1-α) * negative_events
- Fallback to universal AF if negative events insufficient
- Configurable af_weight (default: 0.7)
- Fix PNG output path in visualization example
- Move generated plots to gates/examples/synthetic_plots/
- Create example to visualize synthetic FCS scenarios using flow-plots
- Generate scatter plots for all 4 test scenarios
- Document decision to keep synthetic FCS generation in crate-specific test helpers
- Avoid cyclic dependency risk by not moving to flow-utils
- Add K-means clustering method for scatter gating
- Add GMM clustering method for scatter gating
- Identify main population cluster/component
- Generate ellipse gates around main population
- Support multi-population detection structure
- Implement KernelDensity2D for 2D scatter plot density estimation
- Use 2D FFT convolution for efficient computation
- Add contour extraction at density thresholds
- Update scatter gating to use 2D KDE for better density contours
- Generate polygon gates from density contours
- Add compare_doublet_methods for head-to-head comparison
- Add compare_with_peacoqc convenience function
- Calculate agreement matrix between methods
- Recommend method based on agreement and performance
- Supports performance vs accuracy tradeoff analysis
- Add DoubletGateConfig and DoubletMethod enums
- Implement ratio-based MAD method (peacoqc-rs compatible)
- Implement density-based detection using KDE
- Add hybrid method combining multiple approaches
- Support for multiple channel pairs (FSC-A/FSC-H, FSC-W/FSC-H, etc.)
- Note: Clustering method pending linfa API fix
- Add ScatterGateConfig and ScatterGateMethod enums
- Implement density contour and ellipse fit methods
- Add support for multi-population detection (placeholder)
- Integrate with flow-utils KDE for density estimation
- Add interactive pipeline support with user review breakpoints
- Note: Clustering methods pending linfa API fix

### Bug Fixes

<csr-id-3683d6a9248108834f3be9c6ae7a844d96953b7a/>
<csr-id-a1894b8dd78f86970311dde59e0f863a685ef4ec/>
<csr-id-28677b4de7abaccf198f2a278a38c46a2364f193/>
<csr-id-c8d5ab0e62038fc07f17ffb89e9748c3a159007e/>
<csr-id-6596ed9f6d7916684d38ae65f9284ae7a40a937f/>
<csr-id-38013b28d81af8510a1065745d203bd5e2057518/>
<csr-id-ec337c29858cd506aec01548d0e8431fa6eec9f3/>
<csr-id-7b87699eb278bd7b7d37076aaaa730ff99fc3c53/>
<csr-id-383b476374a707447e655b1b0c0a298e91fd2cc3/>
<csr-id-385c0be364793819279fb9a50f38eb29bbceeab3/>
<csr-id-d33d3616c82ffc04001363ad3f3a9b7ccef0175f/>
<csr-id-161b1334a4a20d5fb0be80aee8134732840e9a6a/>

 - <csr-id-465089a6a99336556e492a02b06757fff54fbb63/> update example generation functions to use Gaussian distributions
   - Replace uniform random (rng.gen_range) with Normal distributions in all functions
- Update generate_single_population, generate_multi_population, generate_with_doublets, generate_noisy_data
- Increase main population density for with_debris (10% debris, 90% main, 30k events)
- All scenarios now generate realistic Gaussian/Poisson-like distributions
- Fixes square/rectangular distribution issue in plots
- Fix auto_gate dereference in function call
- Regenerate synthetic FCS plots with latest code
- All compilation errors resolved
- Add explicit f64->f32 casts for Normal distribution samples
- Fix ambiguous numeric type errors
- All generation functions now compile correctly
- Maintains realistic Gaussian distributions for all scenarios
- Change rand from 0.9 to 0.8 to match rand_distr 0.4 requirements
- Fixes compilation errors in test_helpers.rs
- All generation functions now use Gaussian distributions correctly
- Add WithDebris to TestScenario enum
- Implement generate_with_debris function with 15% debris near origin
- Update match statement to include WithDebris
- All generation functions now use Gaussian distributions via rand_distr
- Fix width/height types (u32 instead of i32)
- Use correct builder pattern for axis options
- Import Plot trait for render method
- Add fit_from_rows helper methods to KMeans and GMM
- Convert Array2 from ndarray 0.17 to Vec<Vec<f64>> for compatibility
- Resolve type mismatch between flow-gates (ndarray 0.17) and flow-utils (ndarray 0.16)
- Accidentally removed in cleanup, restore for compilation
- Extract recommended_method before moving results into return value
- All compilation errors resolved
- Remove unnecessary clone of method_name
- Add check for empty methods vector
- All compilation errors resolved
- GateHierarchy tracks relationships, not gates themselves
- Remove incorrect add_gate calls
- Gates are stored in result structs
- Hierarchy can be used to track parent-child relationships if needed
- Fix create_ellipse_geometry calls to use Vec<(f32, f32)> format
- Fix anyhow::Error conversion (doesn't implement StdError)
- Fix GateStatistics::empty access (create empty stats manually)
- All compilation errors resolved
- Use get_parameter_events_slice instead of get_channel_f64
- Convert f32 to f64 for processing
- Fix error handling to use GateError::Other instead of non-existent variants
- All compilation errors resolved

### Test

 - <csr-id-bea47e8ee97b86a3120b8097d0fdbe6bc9fce133/> add synthetic FCS file generation for automated gating tests
   - Create test_helpers module with synthetic data generation
   - Support multiple test scenarios: single population, multi-population, doublets, noisy
   - Generate realistic scatter patterns (FSC-A, FSC-H, FSC-W, SSC-A, SSC-H)
   - Remove #[ignore] from all automated gating tests
   - Enable full test suite execution
 - <csr-id-dcf9154b305c79728dd2a9f61e4440b5a15756ea/> add integration tests for automated gating
   - Add tests for scatter gating (ellipse fit, density contour)
   - Add tests for doublet detection (MAD, density-based)
   - Add tests for preprocessing pipeline (automated and interactive)
   - Tests marked with #[ignore] until test data is available

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 36 commits contributed to the release over the course of 24 calendar days.
 - 24 days passed between releases.
 - 33 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update changelogs prior to release ([`089feff`](https://github.com/jrmoynihan/flow/commit/089feff624625a5ddf0b1da570e4f60b6fedf09b))
    - Update dependencies and align workspace configurations ([`46bee42`](https://github.com/jrmoynihan/flow/commit/46bee42d4f28d185b38446c0d950c2579c422f43))
    - Add hierarchy support and gating improvements ([`42b4620`](https://github.com/jrmoynihan/flow/commit/42b46207448be5ca137b0b1067ddaa1222b50ccb))
    - Triple event counts and adjust distributions ([`0e1ee96`](https://github.com/jrmoynihan/flow/commit/0e1ee96078a18b06ce5c0c8776df9892d7861ea8))
    - Update example generation functions to use Gaussian distributions ([`465089a`](https://github.com/jrmoynihan/flow/commit/465089a6a99336556e492a02b06757fff54fbb63))
    - Fix auto_gate parameter passing and regenerate plots ([`3683d6a`](https://github.com/jrmoynihan/flow/commit/3683d6a9248108834f3be9c6ae7a844d96953b7a))
    - Complete synthetic data generation with debris scenario ([`5996edc`](https://github.com/jrmoynihan/flow/commit/5996edc676f6a606fcd48e2ffc8ed3131f08ce0b))
    - Resolve type inference issues in Gaussian distributions ([`a1894b8`](https://github.com/jrmoynihan/flow/commit/a1894b8dd78f86970311dde59e0f863a685ef4ec))
    - Resolve rand version mismatch and complete example ([`28677b4`](https://github.com/jrmoynihan/flow/commit/28677b4de7abaccf198f2a278a38c46a2364f193))
    - Complete migration to Gaussian distributions for synthetic data ([`547c2ae`](https://github.com/jrmoynihan/flow/commit/547c2ae09f0f263314de70750b8c8e01b4fd4661))
    - Add WithDebris scenario and complete Gaussian distribution migration ([`c8d5ab0`](https://github.com/jrmoynihan/flow/commit/c8d5ab0e62038fc07f17ffb89e9748c3a159007e))
    - Improve synthetic data generation with realistic distributions ([`c0ba8e7`](https://github.com/jrmoynihan/flow/commit/c0ba8e72f6866bda5d9eec40a6f089ccc7c35107))
    - Implement peak biasing and negative event extraction ([`3409773`](https://github.com/jrmoynihan/flow/commit/340977390c10a31fdf7694ac9325147f406c5b72))
    - Correct flow-plots API usage in visualization example ([`6596ed9`](https://github.com/jrmoynihan/flow/commit/6596ed9f6d7916684d38ae65f9284ae7a40a937f))
    - Add visualization example for synthetic test data ([`6a65bd7`](https://github.com/jrmoynihan/flow/commit/6a65bd7077b2a12670c3766248b08447e92ea8b5))
    - Add synthetic FCS file generation for automated gating tests ([`bea47e8`](https://github.com/jrmoynihan/flow/commit/bea47e8ee97b86a3120b8097d0fdbe6bc9fce133))
    - Resolve ndarray version mismatch for clustering ([`38013b2`](https://github.com/jrmoynihan/flow/commit/38013b28d81af8510a1065745d203bd5e2057518))
    - Implement clustering-based scatter gating ([`43a00f6`](https://github.com/jrmoynihan/flow/commit/43a00f6f0e4043d9b973eb8c9ae2c18ff64b780d))
    - Add 2D KDE for improved density contours ([`c89944b`](https://github.com/jrmoynihan/flow/commit/c89944be9c68a1f688dfb5ee333c7562b28f90b1))
    - Restore Gate import in doublets module ([`ec337c2`](https://github.com/jrmoynihan/flow/commit/ec337c29858cd506aec01548d0e8431fa6eec9f3))
    - Clean up unused imports and variables ([`c987a22`](https://github.com/jrmoynihan/flow/commit/c987a225570c2afae480800327d0072ab4b4e4ad))
    - Add implementation status document ([`69d65c9`](https://github.com/jrmoynihan/flow/commit/69d65c959a392f16431cc98beae9c361ccfed10a))
    - Add comprehensive documentation for flow-utils and research notes ([`6f6d0f5`](https://github.com/jrmoynihan/flow/commit/6f6d0f59369453e3f0018b37f1377b204b023223))
    - Fix final borrow checker error ([`7b87699`](https://github.com/jrmoynihan/flow/commit/7b87699eb278bd7b7d37076aaaa730ff99fc3c53))
    - Fix borrow checker error in comparison module ([`383b476`](https://github.com/jrmoynihan/flow/commit/383b476374a707447e655b1b0c0a298e91fd2cc3))
    - Fix GateHierarchy API usage ([`385c0be`](https://github.com/jrmoynihan/flow/commit/385c0be364793819279fb9a50f38eb29bbceeab3))
    - Fix ellipse geometry creation and error handling ([`d33d361`](https://github.com/jrmoynihan/flow/commit/d33d3616c82ffc04001363ad3f3a9b7ccef0175f))
    - Fix Fcs API usage in automated gating ([`161b133`](https://github.com/jrmoynihan/flow/commit/161b1334a4a20d5fb0be80aee8134732840e9a6a))
    - Add doublet detection method comparison ([`7b65fbc`](https://github.com/jrmoynihan/flow/commit/7b65fbcc9119762ee4cf64cf129c017ece95ff30))
    - Add README for automated gating module ([`f9eef00`](https://github.com/jrmoynihan/flow/commit/f9eef00689d5c1dbda8bce37ca0d399afae19d46))
    - Add integration tests for automated gating ([`dcf9154`](https://github.com/jrmoynihan/flow/commit/dcf9154b305c79728dd2a9f61e4440b5a15756ea))
    - Add enhanced doublet detection module ([`c998c06`](https://github.com/jrmoynihan/flow/commit/c998c06382ec30a870452083b7366a74ced5830e))
    - Add automated scatter gating module ([`6762e5f`](https://github.com/jrmoynihan/flow/commit/6762e5f0d484be7e8d45363205793a50e46b0eb3))
    - Merge pull request #10 from jrmoynihan/gpu-acceleration ([`69363eb`](https://github.com/jrmoynihan/flow/commit/69363eb3a664b1aa6cd0be9b980ec08fc03b7955))
    - Release flow-fcs v0.2.0, safety bump 4 crates ([`cd26a89`](https://github.com/jrmoynihan/flow/commit/cd26a8970fc25dbe70c1cc9ac342b367613bcda6))
    - Adjusting changelogs prior to release of flow-fcs v0.1.6 ([`7fb88db`](https://github.com/jrmoynihan/flow/commit/7fb88db9ede05b317a03d367cea18a3b8b73c5a1))
</details>

<csr-unknown>
 add comprehensive documentation for flow-utils and research notes add README for automated gating module triple event counts and adjust distributions complete synthetic data generation with debris scenario complete migration to Gaussian distributions for synthetic data improve synthetic data generation with realistic distributions implement peak biasing and negative event extractionTask 2.2: Peak BiasingTask 2.3: Extract Negative Events from ControlsTask 2.4: Hybrid Autofluorescence (partial) add visualization example for synthetic test data implement clustering-based scatter gating add 2D KDE for improved density contours add doublet detection method comparison add enhanced doublet detection module add automated scatter gating module fix auto_gate parameter passing and regenerate plots resolve type inference issues in Gaussian distributions resolve rand version mismatch and complete example add WithDebris scenario and complete Gaussian distribution migration correct flow-plots API usage in visualization example resolve ndarray version mismatch for clustering restore Gate import in doublets module fix final borrow checker error fix borrow checker error in comparison module fix GateHierarchy API usage fix ellipse geometry creation and error handling fix Fcs API usage in automated gating<csr-unknown/>

## 0.1.2 (2026-01-21)

<csr-id-e670a9216137c9a2cedde38f3e21894f280fe516/>
<csr-id-a0b4bcdd64294de3a0e40795c6db838cbcb18ac0/>
<csr-id-4bbcfad61b695c86b6b07173486e5580d8b9eeae/>

### New Features

<csr-id-7018701b741c6910e89c93e21ca4249120a1eb1b/>
<csr-id-873cfaee2af2b444fe0cd951ed701fade83febc0/>
<csr-id-b6bf3fcdc9e7466c234ecd30b47db57abc52f643/>
<csr-id-d2068182f96d737d1febfca6854ad89d84a6cbfe/>
<csr-id-e8455560b2f20ff0dda711f866f5eaf71d1d323d/>

 - <csr-id-2b7981fa03249f2052e4078ca6b145371c1a661c/> expand error types for new features
   Add comprehensive error types to support new functionality.
   
   - Add HierarchyCycle error for cycle detection

### Refactor

 - <csr-id-e670a9216137c9a2cedde38f3e21894f280fe516/> update module structure after GPU removal
   - Remove gpu module from lib.rs
   - Update all GPU references to use batch_filtering module
   - Simplify conditional compilation by removing GPU feature flags
 - <csr-id-a0b4bcdd64294de3a0e40795c6db838cbcb18ac0/> remove GPU implementation, use CPU-only batch filtering
   - Remove all GPU code (backend, filter, kernels)
   - Create new batch_filtering module with optimized CPU implementation
   - Remove GPU dependencies (burn, cubecl) from Cargo.toml
   - Update types.rs and filtering/mod.rs to use batch_filtering directly
   - Add GPU_PERFORMANCE_FINDINGS.md documenting why GPU was removed
   - GPU was 2-10x slower than CPU at all batch sizes due to overhead
 - <csr-id-4bbcfad61b695c86b6b07173486e5580d8b9eeae/> update library exports and documentation
   Update public API exports to include new features and improve
   documentation.
   
   - Export GateLinks, GateQuery, and new filtering functions
   - Export BooleanOperation and GateBuilder
   - Export gate geometry traits (GateBounds, GateCenter, etc.)
   - Export GatingML import/export functions
   - Add ParameterSet type alias
   - Update documentation examples to be compilable
   - Fix example code formatting

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release.
 - 3 days passed between releases.
 - 9 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Adjusting changelogs prior to release of flow-fcs v0.1.5, flow-plots v0.1.3, flow-gates v0.1.2 ([`9c8f44a`](https://github.com/jrmoynihan/flow/commit/9c8f44a6b5908a262825a2daa8b3963fdea99a11))
    - Release flow-fcs v0.1.5, flow-gates v0.1.2 ([`4106abc`](https://github.com/jrmoynihan/flow/commit/4106abc5ae2d35328ec470daf9b0a9a549ebd6ba))
    - Update module structure after GPU removal ([`e670a92`](https://github.com/jrmoynihan/flow/commit/e670a9216137c9a2cedde38f3e21894f280fe516))
    - Remove GPU implementation, use CPU-only batch filtering ([`a0b4bcd`](https://github.com/jrmoynihan/flow/commit/a0b4bcdd64294de3a0e40795c6db838cbcb18ac0))
    - Merge pull request #9 from jrmoynihan/flow-gates ([`d6e993e`](https://github.com/jrmoynihan/flow/commit/d6e993ea8eb206c676aa0a95d01fc8cfaec882c9))
    - Update library exports and documentation ([`4bbcfad`](https://github.com/jrmoynihan/flow/commit/4bbcfad61b695c86b6b07173486e5580d8b9eeae))
    - Expand error types for new features ([`2b7981f`](https://github.com/jrmoynihan/flow/commit/2b7981fa03249f2052e4078ca6b145371c1a661c))
    - Add gate query builder and filtering helpers ([`7018701`](https://github.com/jrmoynihan/flow/commit/7018701b741c6910e89c93e21ca4249120a1eb1b))
    - Enhance gate hierarchy with reparenting and cloning ([`873cfae`](https://github.com/jrmoynihan/flow/commit/873cfaee2af2b444fe0cd951ed701fade83febc0))
    - Add boolean gate support to GatingML import/export ([`b6bf3fc`](https://github.com/jrmoynihan/flow/commit/b6bf3fcdc9e7466c234ecd30b47db57abc52f643))
    - Add boolean gate support ([`d206818`](https://github.com/jrmoynihan/flow/commit/d2068182f96d737d1febfca6854ad89d84a6cbfe))
    - Add gate linking system ([`e845556`](https://github.com/jrmoynihan/flow/commit/e8455560b2f20ff0dda711f866f5eaf71d1d323d))
</details>

<csr-unknown>
Add InvalidBooleanOperation error for boolean gate validationAdd GateNotFound error for missing gate referencesAdd InvalidLink error for gate linking operationsAdd CannotReparent error for hierarchy operationsAdd InvalidSubtreeOperation error for subtree operationsAdd EmptyOperands error for boolean operationsAdd InvalidBuilderState error for builder validationAdd DuplicateGateId error for ID conflictsAdd helper constructors for all new error typesAdd GateQuery builder with fluent APIAdd filter_gates_by_parameters() helperAdd filter_gates_by_scope() helperAdd filter_hierarchy_by_parameters() helperSupport filtering by parameters, scope, and typeImprove documentation and examplesAdd reparent() to move a gate to a new parentAdd reparent_subtree() to move entire subtreesAdd clone_subtree() to duplicate subtrees with new IDsAdd cycle detection to prevent invalid hierarchiesImprove error handling with specific error typesAdd write_boolean_gate for exporting boolean gates to XMLAdd parse_boolean_gate_v1_5 and parse_boolean_geometry_v2 for importSupport AND, OR, and NOT operations in GatingMLReplace anyhow::Result with custom GateError::ResultImprove error handling with custom error typesAdd BooleanOperation enum (And, Or, Not)Add Boolean variant to GateGeometry with operation and operandsAdd GateResolver trait for resolving gate IDs to gate referencesImplement boolean gate filtering with filter_events_booleanAdd filter_by_gate_with_resolver for boolean gate supportUpdate EventIndex to handle boolean gates via resolverAdd GateLinks with add_link, remove_link, get_links methodsTrack which gates reference other gatesSupport querying link counts and checking if gates are linked<csr-unknown/>

## 0.1.1 (2026-01-18)

<csr-id-d3aa6cdc5a806703131a3ffac63506142f052da9/>
<csr-id-8d232b2838f65aa621a81031183d4c954d787543/>
<csr-id-4649c7af16150d05880ddab4e732e9dee374d01b/>
<csr-id-fbbef211ba3c7f4dffa75ea7d56f65e249e72384/>

### Chore

 - <csr-id-d3aa6cdc5a806703131a3ffac63506142f052da9/> update Cargo.toml scripts and dependency versions
   - Standardize version formatting for flow-fcs dependencies across multiple Cargo.toml files.
   - Update dry-release, publish, and changelog scripts to include specific package names for clarity.
 - <csr-id-8d232b2838f65aa621a81031183d4c954d787543/> update publish command in Cargo.toml files to include --update-crates-index
 - <csr-id-4649c7af16150d05880ddab4e732e9dee374d01b/> update Cargo.toml files for consistency and improvements
   - Standardize formatting in Cargo.toml files across multiple crates
   - Update repository URLs to reflect new structure
   - Enhance keywords and categories for better discoverability
   - Ensure consistent dependency declarations and script commands

### Other

 - <csr-id-fbbef211ba3c7f4dffa75ea7d56f65e249e72384/> :arrow_up: bump quick-xml version

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release over the course of 4 calendar days.
 - 4 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release flow-plots v0.1.2, flow-gates v0.1.1 ([`2c36741`](https://github.com/jrmoynihan/flow/commit/2c367411265c8385e88b2653e278bd1e2d1d2198))
    - Release flow-fcs v0.1.4, peacoqc-rs v0.1.2 ([`140a59a`](https://github.com/jrmoynihan/flow/commit/140a59af3c1ca751672e66c9cc69708f45ac8453))
    - Release flow-fcs v0.1.3, peacoqc-rs v0.1.2 ([`607fcae`](https://github.com/jrmoynihan/flow/commit/607fcae78304d51ce8d156e82e5dba48a1b6dbfa))
    - Update Cargo.toml scripts and dependency versions ([`d3aa6cd`](https://github.com/jrmoynihan/flow/commit/d3aa6cdc5a806703131a3ffac63506142f052da9))
    - Release flow-fcs v0.1.3 ([`e79b57f`](https://github.com/jrmoynihan/flow/commit/e79b57f8fd7613fbdcc682863fef44178f14bed8))
    - Update publish command in Cargo.toml files to include --update-crates-index ([`8d232b2`](https://github.com/jrmoynihan/flow/commit/8d232b2838f65aa621a81031183d4c954d787543))
    - Merge pull request #8 from jrmoynihan/peacoqc-rs ([`fbeaab2`](https://github.com/jrmoynihan/flow/commit/fbeaab262dc1a72832dba3d6c4708bf95c941929))
    - Merge branch 'main' into peacoqc-rs ([`c52af3c`](https://github.com/jrmoynihan/flow/commit/c52af3c09ae547a7e1ce2c62e9999590314e8f97))
    - Update Cargo.toml files for consistency and improvements ([`4649c7a`](https://github.com/jrmoynihan/flow/commit/4649c7af16150d05880ddab4e732e9dee374d01b))
    - :arrow_up: bump quick-xml version ([`fbbef21`](https://github.com/jrmoynihan/flow/commit/fbbef211ba3c7f4dffa75ea7d56f65e249e72384))
</details>

## 0.1.0 (2026-01-14)

<csr-id-5f63c2c2f02f2abaa1862153743e1923c71d8d86/>
<csr-id-fd12ce3ff00c02e75c9ea84848adb58b32c4d66f/>
<csr-id-f64872e441add42bc9d19280d4411df628ff853e/>
<csr-id-d14cd7b41828c45396709071065c98d9bda5c967/>
<csr-id-621d3aded59ff51f953c6acdb75027c4541a8b97/>
<csr-id-f0f0ab21b68eb1a28903957bae137f326b5a082b/>

### Chore

 - <csr-id-5f63c2c2f02f2abaa1862153743e1923c71d8d86/> add GatingML 2.0 Specification PDF for reference
 - <csr-id-fd12ce3ff00c02e75c9ea84848adb58b32c4d66f/> reorganize workspace into separate crates

### Chore

 - <csr-id-f0f0ab21b68eb1a28903957bae137f326b5a082b/> Update CHANGELOG for upcoming release
   - Documented version bump, enhancements in FCS file parsing, benchmarking capabilities, and metadata processing improvements.
   - Updated plotting backend and TypeScript bindings for pixel data.
   - Refactored folder names for better organization.

### Chore

 - <csr-id-621d3aded59ff51f953c6acdb75027c4541a8b97/> update CHANGELOG for upcoming release
   - Documented unreleased changes including version bump, enhancements in FCS file parsing, benchmarking capabilities, and metadata processing improvements.
   - Updated plotting backend and TypeScript bindings for pixel data.
   - Refactored folder names for better organization and removed unused imports.

### New Features

 - <csr-id-7a1233b4426b5c7b5849666b28b75a3bee19e8c7/> introduce flow-gates library for flow cytometry data analysis
   - Added core functionality for creating and managing gates, including Polygon, Rectangle, and Ellipse geometries.

### Refactor

 - <csr-id-f64872e441add42bc9d19280d4411df628ff853e/> :truck: Rnamed folders without the `flow-` prefix.
   Just shorter to type paths.  We'll keep the crates named with the `flow-` prefix when we publish.

### Test

 - <csr-id-d14cd7b41828c45396709071065c98d9bda5c967/> :white_check_mark: Add GatingML compliance test files
   Added readme, test text, fcs, and xml files to parse and validate

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 13 commits contributed to the release over the course of 7 calendar days.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release flow-gates v0.1.0 ([`869b4c2`](https://github.com/jrmoynihan/flow/commit/869b4c2f123ef2ebbf5a464b4453a71f35a6ad06))
    - Remove extra keywords ([`fbf2fa6`](https://github.com/jrmoynihan/flow/commit/fbf2fa66dbee6a2d6c188a8b9a7f933ca3d2929b))
    - Release flow-plots v0.1.1, flow-gates v0.1.0 ([`b5be6ba`](https://github.com/jrmoynihan/flow/commit/b5be6ba4e2093a8b0e972bd44265fa51b8c6be13))
    - Update CHANGELOG for upcoming release ([`f0f0ab2`](https://github.com/jrmoynihan/flow/commit/f0f0ab21b68eb1a28903957bae137f326b5a082b))
    - Release flow-fcs v0.1.2 ([`57f4eb7`](https://github.com/jrmoynihan/flow/commit/57f4eb7de85c2b41ef886db446f63d753c5faf05))
    - Update CHANGELOG for upcoming release ([`621d3ad`](https://github.com/jrmoynihan/flow/commit/621d3aded59ff51f953c6acdb75027c4541a8b97))
    - Merge branch 'main' into flow-gates ([`4d40ba1`](https://github.com/jrmoynihan/flow/commit/4d40ba1bfa95f9df97a3dbfcc3c22c9bf701a5dd))
    - Merge branch 'flow-gates' into main ([`c2f2d13`](https://github.com/jrmoynihan/flow/commit/c2f2d13a61854f93687cdfd2f6a1b4b12e0d9810))
    - :truck: Rnamed folders without the `flow-` prefix. ([`f64872e`](https://github.com/jrmoynihan/flow/commit/f64872e441add42bc9d19280d4411df628ff853e))
    - Introduce flow-gates library for flow cytometry data analysis ([`7a1233b`](https://github.com/jrmoynihan/flow/commit/7a1233b4426b5c7b5849666b28b75a3bee19e8c7))
    - Add GatingML 2.0 Specification PDF for reference ([`5f63c2c`](https://github.com/jrmoynihan/flow/commit/5f63c2c2f02f2abaa1862153743e1923c71d8d86))
    - :white_check_mark: Add GatingML compliance test files ([`d14cd7b`](https://github.com/jrmoynihan/flow/commit/d14cd7b41828c45396709071065c98d9bda5c967))
    - Reorganize workspace into separate crates ([`fd12ce3`](https://github.com/jrmoynihan/flow/commit/fd12ce3ff00c02e75c9ea84848adb58b32c4d66f))
</details>

