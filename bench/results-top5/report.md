## Entorno

| | |
|---|---|
| Fecha | 2026-09-29 14:22 |
| CPU | AMD Ryzen 7 2700 Eight-Core Processor (16 hilos) |
| Kernel | 7.0.0-34-generic |
| Modo de frecuencia | schedutil |
| Docker | 29.1.3 |
| Imagen Rust | 45d863ea829d (159MB) |
| Imagen Java | 50aadff76693 (579MB) |
| Java | openjdk version "25.0.4.1" 2026-08-18 LTS |
| k6 | k6 v2.3.0 (commit/e088784614, go1.27.1, linux/amd64) |
| Núcleos | motor `4,6`, NATS `2,3`, k6 `8-15` |
| Combinaciones por respuesta | hasta 5 |
| Repeticiones | 3, orden alternado; 30s por tasa, 5 s de pausa entre tasas |

## Arranque y reposo (mediana)

| | Rust | Java |
|---|---|---|
| Arranque hasta `/ready` (ms) | 519 | 3930 |
| Memoria en reposo con catálogo (MiB) | 5 | 151 |
| CPUs detectadas por el motor | 2 | 2 |

## Primeras solicitudes en frío (200/s, 20 s)

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 200 | 0.42 | 0.46 | 0.53 | 0.52–0.60 | 2.87 | 0.0 % | 0 | 6 % | 8 |
| java | 200 | 1.06 | 2.26 | 3.83 | 3.59–3.93 | 13 | 0.0 % | 0 | 48 % | 183 |

## 1000 solicitudes/s

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 1000 | 0.35 | 0.47 | 0.57 | 0.55–0.57 | 7.39 | 0.0 % | 0 | 22 % | 13 |
| java | 1000 | 0.47 | 0.75 | 1.61 | 1.60–1.70 | 11 | 0.0 % | 0 | 50 % | 193 |

## 2000 solicitudes/s

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 2000 | 0.34 | 0.54 | 0.69 | 0.62–0.88 | 11 | 0.0 % | 0 | 40 % | 21 |
| java | 2000 | 0.41 | 0.53 | 1.55 | 1.51–1.58 | 9.80 | 0.0 % | 0 | 60 % | 207 |

## 3000 solicitudes/s

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 3000 | 0.31 | 0.42 | 0.75 | 0.72–0.81 | 12 | 0.0 % | 0 | 49 % | 30 |
| java | 3000 | 0.28 | 0.52 | 1.63 | 1.52–1.65 | 12 | 0.0 % | 0 | 56 % | 231 |

## 4000 solicitudes/s

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 4000 | 0.31 | 0.46 | 1.31 | 1.30–2.46 | 18 | 0.0 % | 0 | 68 % | 39 |
| java | 4000 | 0.27 | 0.64 | 2.92 | 2.74–3.02 | 25 | 0.0 % | 0 | 67 % | 260 |

## 6000 solicitudes/s

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 5999 | 0.25 | 0.45 | 2.07 | 1.83–2.31 | 20 | 0.0 % | 0 | 78 % | 49 |
| java | 5999 | 0.25 | 0.88 | 3.95 | 3.88–4.77 | 21 | 0.0 % | 0 | 83 % | 302 |

## 8000 solicitudes/s

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 7999 | 0.22 | 0.40 | 2.92 | 2.83–3.20 | 38 | 0.0 % | 0 | 83 % | 62 |
| java | 7999 | 0.24 | 1.26 | 9.82 | 7.52–14 | 207 | 0.0 % | 0 | 112 % | 360 |

Mediana de 3 repeticiones. CPU sobre 100 % por núcleo asignado.
