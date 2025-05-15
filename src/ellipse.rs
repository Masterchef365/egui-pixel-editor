pub fn ellipse(wx: isize, wy: isize, x: isize, y: isize) -> bool {
    let x2 = x * x;
    let y2 = y * y;
    let wx2 = wx * wx;
    let wy2 = wy * wy;

    y2 * wx2 <= wy2 * wx2 - wy2 * x2
}



pub fn solve_ellipse(wx: isize, wy: isize, x: isize) -> isize {
    assert!(wx >= 0);
    assert!(wy >= 0);
    assert!(x <= wx);
    assert!(x >= -wx);
    
    let mut min = 0;
    let mut max = wy;

    // NOTE: Should be logarithmic!
    for _ in 0..=isize::BITS {
        let mid = (max + min) / 2;
        let check_min = ellipse(wx, wy, x, min);
        let check_max = ellipse(wx, wy, x, max);
        let check_mid = ellipse(wx, wy, x, mid);
        
        if check_max {
            return max;
        }
        
        if max - min == 1 && check_min {
            return min;
        }
        
        if check_mid {
            min = mid;
        } else {
            max = mid;
        }
    }

    unreachable!("Ellipse went unsolved!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn check_solve_ellipse(f: fn(isize, isize, isize) -> isize) {
        let n = 50;
        for wx in 0..=n {
            for wy in 0..=n {
                for x in -wx..=wx {
                    for y in -wx..=wx {
                        assert!(
                            ellipse(wx, wy, x, y) == (y.abs() <= f(wx, wy, x)),
                            "x={x} y={y} wx={wx} wy={wy}"
                        )
                    }
                }
            }
        }
    }

    fn solve_ellipse_naive(wx: isize, wy: isize, x: isize) -> isize {
        assert!(wx >= 0);
        assert!(wy >= 0);
        assert!(x <= wx);
        assert!(x >= -wx);

        let mut y = wy;
        while !ellipse(wx, wy, x, y) && y > 0 {
            y -= 1;
        }
        y
    }

    #[test]
    fn test_solve_ellipse_naive() {
        check_solve_ellipse(solve_ellipse_naive)
    }

    #[test]
    fn test_solve_ellipse() {
        check_solve_ellipse(solve_ellipse)
    }

}
