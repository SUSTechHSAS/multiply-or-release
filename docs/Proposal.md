# Project Title

Team members:

- Name1
- Name2
- Name3

## Summary Description

A summary description of the overall goal of the project.

## Additional Details

- One or more typical “use cases”. These might include “storyboards” explaining
  how a user would interact with the program or some interesting “input/output”
  examples.
- A sketch of intended components (key functions, key data structures, separate
  modules).
- Thoughts on testing. These might include critical functions or data structures
  that will be given `#[test]` functions. Also consider using the
  [`test_case`](https://crates.io/crates/test-case) crate,
  [`quickcheck`](https://crates.io/crates/quickcheck) crate,
  [`proptest`](https://crates.io/crates/proptest) crate, or [`cargo
  fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html) tool.
- Thoughts on a “minimal viable product” and “stretch goals”. Be sure to review
  the final project grading rubric and consider organizing the project around a
  core deliverable that will almost certainly be achieved and then a number of
  extensions and features that could be added to ensure that project is of
  suitable size/scope/effort.
- Expected functionality to be completed at the Checkpoint.

***
***

The following should be deleted from the proposal document, but is included in the initial `Proposal.md` file for reference.

## Final Project Rubric

- Correctness (15%)
  - Free of compilation errors and warnings.
  - Correctly accomplishes the goals of the project.
  - Correctness is supported by suitable testing.
- Style/Design (15%)
  - Project applies one or more elements of Rust design:
    - Uses traits.  Minimally, makes use of traits from the Rust Standard Library (e.g., `PartialEq` and `Iterator`).  Better, implements appropriate traits for the types defined for the project.  Best, defines and   uses novel traits for the project.
    - Uses `struct` and `enum` definitions appropriately.
    - Uses types to capture invariants.
    - Uses modules appropriately (e.g., place distinct data structures in distinct modules).
- Effort/Accomplishment (30%)
  - How “big” is the project?
    - A “small” project will have at least 500 Lines of Rust Code per team member.  (Significantly less than that corresponds to a “tiny” project and would not be acceptable for this activity.)  A “medium” or “large” project may have significantly more and would likely also correspond to a more “difficult” project (see below).
  - How “difficult” was the project?
    - An “easy” project that required trivial data structures and algorithms. Feedback about how to extend the project was ignored.  (Projects falling into this category are likely to lose points in Style/Design as well.)
    - A “moderate” project that applied basic course concepts, but did not require the group members to significantly challenge themselves or to learn something new.
    - A “challenging” project that demonstrates significant thought in design and implementation.  Clear that the group members challenged themselves and learned something new by undertaking the project.
  - What work was done for the project?  (This includes both the work embodied by the final submission and work not obvious from the final submission (e.g., approaches attempted but then abandoned, suitably described).) 
  - Did the project require learning advanced features?
  - Did all team members contribute to the project?
- Presentation (10\%)
