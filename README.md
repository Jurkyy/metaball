# Metaball

ASCII metaball animation for the terminal, written in Rust.

![metaball](https://img.shields.io/badge/rust-stable-orange)

## Run

```bash
cargo run --release
```

## The Math Behind Metaballs

Metaballs are an elegant application of **implicit surfaces** and **scalar fields** in computer graphics.

### Scalar Field Definition

Each blob generates a scalar field based on the inverse-square distance function:

```
f(x, y) = r² / d²
```

Where:
- `r` = blob radius (influence strength)
- `d² = (x - cx)² + (y - cy)²` = squared Euclidean distance from blob center

This creates a smooth falloff: the field is strongest at the center and diminishes with distance.

### Implicit Surface (Isosurface)

The metaball surface is defined as the set of points where the **sum of all field contributions** equals a threshold:

```
Σ fᵢ(x, y) = τ
```

When two blobs approach each other, their fields add together. Points between them that were previously below the threshold can exceed it, causing the characteristic "blobbing" merge effect.

### Field Superposition

The key insight is **linear superposition** - we simply sum the contributions:

```rust
fn calculate_field(&self, x: f64, y: f64) -> f64 {
    self.blobs.iter().map(|b| b.field_at(x, y)).sum()
}
```

This is analogous to:
- Electric potential from multiple point charges
- Gravitational fields from multiple masses
- Gaussian mixture models in statistics

### Aspect Ratio Correction

Terminal characters are taller than wide (~2:1). To render circular blobs, we apply an **anisotropic scaling** to the distance calculation:

```rust
let dx = (px - self.x) / ASPECT_RATIO;  // Scale x by 0.5
let dy = py - self.y;
let dist_sq = dx * dx + dy * dy;
```

This transforms the coordinate space so circles appear circular on screen.

### Edge Detection (Contour Mode)

The contour renderer uses a **discrete gradient approximation** to find the isosurface boundary:

```rust
// Sample 4-connected neighbors
let neighbors = [(row-1, col), (row+1, col), (row, col-1), (row, col+1)];

// Edge exists where inside/outside status changes
for (nr, nc) in neighbors {
    if (field[row][col] >= τ) != (field[nr][nc] >= τ) {
        is_edge = true;
    }
}
```

This is a simplified version of the **marching squares** algorithm.

### Sub-pixel Sampling (Blocks Mode)

For smoother edges, we use **2×2 supersampling**:

```rust
for dy in [0.0, 0.5] {
    for dx in [0.0, 0.5] {
        if field(x + dx, y + dy) >= τ { count += 1; }
    }
}
// Map count ∈ {0,1,2,3,4} → {' ','░','▒','▓','█'}
```

This provides 5 levels of coverage, approximating anti-aliasing in a character cell.

## Render Modes

| Mode | Description |
|------|-------------|
| Gradient | Field intensity mapped to ASCII density |
| Contour | Isosurface boundary detection |
| Solid | Binary threshold with intensity shading |
| Blocks | Unicode blocks with sub-pixel sampling |
| Gooey | Circular chars emphasizing merge zones |

## References

- Blinn, J. (1982). "A Generalization of Algebraic Surface Drawing"
- Nishimura et al. (1985). "Object Modeling by Distribution Function and a Method of Image Generation" (introduced the term "metaball")
