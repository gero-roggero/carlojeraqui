// Dibujo de íconos sin dependencias (lo usa también build.rs para el ícono del .exe).
#![allow(dead_code)]

/// Ícono de estado (bandeja). RGBA sin premultiplicar.
/// Grabando: disco blanco con punto negro. Pausado: anillo gris.
pub fn status_rgba(size: u32, on: bool) -> Vec<u8> {
    let s = size as f32;
    let c = s / 2.0;
    let r_out = s * 0.46;
    let border = (s / 16.0).max(1.0);
    let r_dot = s * 0.17;
    let mut v = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let d = dist(x, y, c);
            let cov = |r: f32| (r - d + 0.5).clamp(0.0, 1.0);
            let (lum, a) = if on {
                let inner = cov(r_out - border);
                let dot = cov(r_dot);
                (20.0 + 225.0 * inner * (1.0 - dot), cov(r_out))
            } else {
                (150.0, cov(r_out) * (1.0 - cov(r_out - border * 2.0)))
            };
            put(&mut v, size, x, y, [lum as u8; 3], a);
        }
    }
    v
}

/// Ícono de la app: azulejo redondeado violeta a rosa con una "C" blanca y un punto de grabación.
pub fn app_rgba(size: u32) -> Vec<u8> {
    let s = size as f32;
    let c = s / 2.0;
    let radius = s * 0.22;
    let half = s * 0.47;
    let ring_out = s * 0.31;
    let ring_in = s * 0.31 - (s * 0.085).max(1.5);
    let dot = s * 0.10;
    // Media abertura de la "C" (radianes), del lado derecho.
    let gap = 0.80f32;
    let (from, to) = ([124.0f32, 58.0, 237.0], [236.0f32, 72.0, 153.0]);
    let mut v = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let px = x as f32 + 0.5 - c;
            let py = y as f32 + 0.5 - c;
            // Distancia a un rectángulo redondeado.
            let qx = (px.abs() - (half - radius)).max(0.0);
            let qy = (py.abs() - (half - radius)).max(0.0);
            let rd = (qx * qx + qy * qy).sqrt() - radius;
            let tile = (0.5 - rd).clamp(0.0, 1.0);
            let d = (px * px + py * py).sqrt();
            let cov = |r: f32| (r - d + 0.5).clamp(0.0, 1.0);
            // La abertura con borde suave: la distancia angular pasada a píxeles.
            let open = ((py.atan2(px).abs() - gap) * d + 0.5).clamp(0.0, 1.0);
            let white = (cov(ring_out) * (1.0 - cov(ring_in)) * open + cov(dot)).min(1.0);
            // Degradé en diagonal, de arriba a la izquierda hacia abajo a la derecha.
            let t = ((px + py) / (2.0 * half) * 0.5 + 0.5).clamp(0.0, 1.0);
            let rgb = [0, 1, 2].map(|i| {
                let base = from[i] + (to[i] - from[i]) * t;
                (base + (250.0 - base) * white) as u8
            });
            put(&mut v, size, x, y, rgb, tile);
        }
    }
    v
}

fn dist(x: u32, y: u32, c: f32) -> f32 {
    let dx = x as f32 + 0.5 - c;
    let dy = y as f32 + 0.5 - c;
    (dx * dx + dy * dy).sqrt()
}

fn put(v: &mut [u8], size: u32, x: u32, y: u32, rgb: [u8; 3], a: f32) {
    let i = ((y * size + x) * 4) as usize;
    v[i..i + 3].copy_from_slice(&rgb);
    v[i + 3] = (a * 255.0).round() as u8;
}
