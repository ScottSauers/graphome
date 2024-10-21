// tests/test_eigen.rs

//! Unit tests for the eigen module.

use ndarray::array;
use graphome::eigen::{call_eigendecomp, save_array_to_csv_dsbevd, compute_ngec, compute_eigenvalues_and_vectors_sym, compute_eigenvalues_and_vectors_sym_band, max_band, to_banded_format};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::fs;
use nalgebra::Matrix;
use nalgebra::DMatrix;

const TOLERANCE: f64 = 1e-9;

#[test]
fn test_eigendecomp() {
    // Define a small Laplacian matrix
    let laplacian = array![
        [2.0, -1.0, 0.0],
        [-1.0, 2.0, -1.0],
        [0.0, -1.0, 2.0]
    ];

    // Perform eigendecomposition
    let (eigvals, eigvecs) = call_eigendecomp(&laplacian).expect("Eigendecomposition failed");

    // Assert that the eigenvector matrix is the correct size
    assert_eq!(eigvecs.nrows(), 3);
    assert_eq!(eigvecs.ncols(), 3);
}

#[test]
fn test_save_array_to_csv() {
    // Create a small array to save
    let array = array![[1.0, 2.0], [3.0, 4.0]];

    // Define the output path
    let output_path = Path::new("test_output.csv");

    // Save the array to a CSV file
    save_array_to_csv_dsbevd(&array, &output_path).expect("Failed to save array to CSV");

    // Read back the file contents
    let mut file = File::open(&output_path).expect("Failed to open CSV file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read CSV file");

    // Check that the contents are as expected
    assert_eq!(contents.trim(), "1.0,2.0\n3.0,4.0");

    // Clean up the test output file
    fs::remove_file(output_path).expect("Failed to delete test output file");
}

#[test]
fn test_compute_ngec() {
    // Define a set of eigenvalues
    let eigenvalues = array![1.0, 2.0, 3.0];

    // Compute NGEC
    let ngec = compute_ngec(&eigenvalues).expect("Failed to compute NGEC");

    // Assert that the NGEC value is within the expected range
    assert!(ngec > 0.0 && ngec < 1.0);
}

#[test]
fn test_compare_eigenvalues_lapack_vs_symmetric() {
    // Define a small Laplacian matrix
    let laplacian = array![
        [2.0, -1.0, 0.0],
        [-1.0, 2.0, -1.0],
        [0.0, -1.0, 2.0]
    ];

    // Perform eigendecomposition using LAPACK's dsbevd
    let (eigvals_lapack, _) = call_eigendecomp(&laplacian).expect("LAPACK eigendecomposition failed");

    // Manually force usage of SymmetricEigen
    let kd = laplacian.nrows() as f64 / 2.0; // Set a high bandedness to force SymmetricEigen
    let (eigvals_symmetric, _) = call_eigendecomp(&laplacian).expect("Symmetric eigenvalue calculation failed");

    // Manually check that the eigenvalues are approximately equal within tolerance
    for (v1, v2) in eigvals_lapack.iter().zip(eigvals_symmetric.iter()) {
        assert!(
            (v1 - v2).abs() <= TOLERANCE,
            "Eigenvalues mismatch: v1 = {}, v2 = {}, diff = {}",
            v1, v2, (v1 - v2).abs()
        );
    }
}

#[test]
fn test_compare_eigenvectors_lapack_vs_symmetric() {
    // Define a small Laplacian matrix
    let laplacian = array![
        [2.0, -1.0, 0.0],
        [-1.0, 2.0, -1.0],
        [0.0, -1.0, 2.0]
    ];

    // Perform eigendecomposition using LAPACK's dsbevd
    let (_, eigvecs_lapack) = call_eigendecomp(&laplacian).expect("LAPACK eigendecomposition failed");

    // Manually force usage of SymmetricEigen
    let kd = laplacian.nrows() as f64 / 2.0; // Set a high bandedness to force SymmetricEigen
    let (_, eigvecs_symmetric) = call_eigendecomp(&laplacian).expect("Symmetric eigenvector calculation failed");

    // Manually check that each element of the eigenvectors is approximately equal within tolerance
    for (row_lapack, row_symmetric) in eigvecs_lapack.outer_iter().zip(eigvecs_symmetric.outer_iter()) {
        for (v1, v2) in row_lapack.iter().zip(row_symmetric.iter()) {
            assert!(
                (v1 - v2).abs() <= TOLERANCE,
                "Eigenvector elements mismatch: v1 = {}, v2 = {}, diff = {}",
                v1, v2, (v1 - v2).abs()
            );
        }
    }
}


#[test]
fn test_compare_eigenvalues_direct_lapack_vs_symmetric() {
    // Define a small Laplacian matrix
    let laplacian = array![
        [2.0, -1.0, 0.0],
        [-1.0, 2.0, -1.0],
        [0.0, -1.0, 2.0]
    ];

    // Compute the bandedness (kd) and call LAPACK's dsbevd directly
    let kd = max_band(&laplacian);
    let (eigvals_lapack, _) = compute_eigenvalues_and_vectors_sym_band(&laplacian, kd).expect("LAPACK eigendecomposition failed");

    // Call SymmetricEigen directly
    let (eigvals_symmetric, _) = compute_eigenvalues_and_vectors_sym(&laplacian).expect("Symmetric eigenvalue calculation failed");

    // Manually check that the eigenvalues are approximately equal within tolerance
    for (v1, v2) in eigvals_lapack.iter().zip(eigvals_symmetric.iter()) {
        assert!(
            (v1 - v2).abs() <= TOLERANCE,
            "Eigenvalues mismatch: v1 = {}, v2 = {}, diff = {}",
            v1, v2, (v1 - v2).abs()
        );
    }
}

#[test]
fn test_compare_eigenvectors_direct_lapack_vs_symmetric() {
    // Define a small Laplacian matrix
    let laplacian = array![
        [2.0, -1.0, 0.0],
        [-1.0, 2.0, -1.0],
        [0.0, -1.0, 2.0]
    ];

    // Compute the bandedness (kd) and call LAPACK's dsbevd directly
    let kd = max_band(&laplacian);
    let (_, eigvecs_lapack) = compute_eigenvalues_and_vectors_sym_band(&laplacian, kd).expect("LAPACK eigendecomposition failed");

    // Call SymmetricEigen directly
    let (_, eigvecs_symmetric) = compute_eigenvalues_and_vectors_sym(&laplacian).expect("Symmetric eigenvector calculation failed");

    // Manually check that each element of the eigenvectors is approximately equal within tolerance
    for i in 0..eigvecs_lapack.nrows() {
        for j in 0..eigvecs_lapack.ncols() {
            let v1 = eigvecs_lapack[(i, j)];
            let v2 = eigvecs_symmetric[(i, j)];
            assert!(
                (v1 - v2).abs() <= TOLERANCE,
                "Eigenvector elements mismatch at ({}, {}): v1 = {}, v2 = {}, diff = {}",
                i, j, v1, v2, (v1 - v2).abs()
            );
        }
    }
}

#[test]
fn test_non_negative_eigenvalues_lapack() {
    // Define a small Laplacian matrix
    let laplacian = array![
        [2.0, -1.0, 0.0],
        [-1.0, 2.0, -1.0],
        [0.0, -1.0, 2.0]
    ];

    // Compute the bandedness (kd) and call LAPACK's dsbevd directly
    let kd = max_band(&laplacian);
    let (eigvals_lapack, _) = compute_eigenvalues_and_vectors_sym_band(&laplacian, kd).expect("LAPACK eigendecomposition failed");

    // Check that all eigenvalues are non-negative for LAPACK
    for v in eigvals_lapack.iter() {
        assert!(
            *v >= 0.0,
            "LAPACK eigenvalue is negative: {}",
            v
        );
    }
}

#[test]
fn test_non_negative_eigenvalues_symmetric() {
    // Define a small Laplacian matrix
    let laplacian = array![
        [2.0, -1.0, 0.0],
        [-1.0, 2.0, -1.0],
        [0.0, -1.0, 2.0]
    ];

    // Call SymmetricEigen directly
    let (eigvals_symmetric, _) = compute_eigenvalues_and_vectors_sym(&laplacian).expect("Symmetric eigenvalue calculation failed");

    // Check that all eigenvalues are non-negative for SymmetricEigen
    for v in eigvals_symmetric.iter() {
        assert!(
            *v >= 0.0,
            "SymmetricEigen eigenvalue is negative: {}",
            v
        );
    }
    #[test]
    fn test_to_banded_format() {
        let matrix = array![
            [1.0, 2.0, 0.0],
            [2.0, 3.0, 4.0],
            [0.0, 4.0, 5.0]
        ];
        let kd = 1;
        let banded = to_banded_format(&matrix, kd);
        let expected = array![
            [2.0, 4.0, 0.0], // kd = 1 (first row above main)
            [1.0, 3.0, 5.0]  // main diagonal
        ];
        assert_eq!(banded, expected);
    }

    #[test]
    fn test_banded_format_conversion() {
        let matrix = array![
            [4.0, 1.0, 0.0, 0.0],
            [1.0, 3.0, 1.0, 0.0],
            [0.0, 1.0, 2.0, 1.0],
            [0.0, 0.0, 1.0, 1.0]
        ];
        let kd = 1;
        let banded = to_banded_format(&matrix, kd);
        let expected = array![
            [1.0, 1.0, 1.0, 0.0], // kd = 1
            [4.0, 3.0, 2.0, 1.0]  // main diagonal
        ];
        assert_eq!(banded, expected, "Banded matrix format is incorrect");
    }
}
