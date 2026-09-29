# El Santuario del Cristal

**Proyecto 2 — Diorama con Raytracing**  
**Autor:** Juan Jose Rivas Alvarez — Carnet 24856

Diorama interactivo inspirado en la lectura visual de las maquetas de aventura. Está construido en Rust y utiliza raylib únicamente para crear la ventana, leer los controles y presentar el framebuffer. La imagen 3D se calcula con un raytracer CPU propio.

## Estado actual

- Intersección analítica rayo–caja mediante el método *slab*.
- Cámara orbital, zoom y rotación automática.
- Framebuffer fijo de 512×288 con cuadros completos, sin barrido entrelazado ni cambios de resolución.
- Personaje voxel de explorador inspirado en Captain Toad, con sombrero, linterna emisiva, mochila, animación de caminata y cámara de seguimiento.
- Escala compacta del personaje ajustada a los pasillos y obstáculos del diorama.
- Colisiones AABB con columnas, ruinas, agua, mecanismos y bordes del diorama.
- Puente ensanchado con descansos abiertos y acceso transitable por peldaños, sin necesidad de salto.
- Rutas de objetivos despejadas y composición por terrazas inspirada en los niveles de Captain Toad.
- Puzzle de tres fragmentos emisivos y pulsantes: al recogerlos se puede activar el cristal central, cambiar su energía y transformar la iluminación de la escena.
- Objetivo guiado mediante HUD, indicadores de progreso y mensajes de descubrimiento.
- Escena modular formada exclusivamente por bloques.
- Santuario ambientado con árboles voxelados, arbustos, faroles, un arco derruido y mampostería dispersa.
- Agua, cascada y cristales con variación animada calculada dentro del material.
- Seis materiales principales procedurales con parámetros independientes, además de materiales del personaje, vegetación y energía.
- Sombras, iluminación especular, niebla atmosférica, viñeta y tone mapping cinematográfico ACES.
- Reflexión recursiva en cobre y cristal.
- Refracción con índice de refracción y efecto Fresnel.
- Skybox procedural de atardecer con sol, nubes amplias y siluetas montañosas por capas.
- Renderizado paralelo usando hilos de la biblioteca estándar de Rust.

## Materiales

| Material | Albedo | Specular | Transparencia | Reflectividad |
|---|---:|---:|---:|---:|
| Piedra con musgo | Gris/verde procedural | 0.10 | 0.00 | 0.03 |
| Arenisca tallada | Naranja con vetas | 0.18 | 0.00 | 0.04 |
| Madera | Marrón con anillos | 0.24 | 0.00 | 0.06 |
| Cobre con pátina | Cobre/turquesa | 0.92 | 0.00 | 0.58 |
| Cristal mágico | Azul | 0.95 | 0.78 | 0.16 |
| Agua | Turquesa con ondas | 0.98 | 0.72 | 0.12 |

## Ejecutar

```bash
cargo run --release
```

Se recomienda usar el perfil `release`, ya que el trazado de rayos se ejecuta en CPU.

La aplicación mantiene siempre un framebuffer de 512×288 y presenta únicamente cuadros completos para evitar mareo o mezcla entre posiciones de cámara. Durante el movimiento usa una muestra directa optimizada; al detenerse conserva la misma resolución y calcula tres muestras con tres rebotes.

Para medir el rendimiento interactivo del raytracer sin depender de la interfaz:

```bash
cargo run --release -- --benchmark
```

Para generar una captura de 960×540 sin abrir la ventana:

```bash
cargo run --release -- --render-preview
```

## Controles

- `WASD`: mover a Toad en relación con la cámara.
- Arrastrar con el botón izquierdo: orbitar la cámara.
- Rueda del mouse: acercar o alejar.
- Flechas izquierda/derecha: orbitar horizontalmente.
- Flechas arriba/abajo: cambiar la elevación de la cámara.
- `F`: activar o desactivar el seguimiento del explorador.
- `Espacio`: activar o detener la rotación automática.
- `R`: reiniciar la cámara, el personaje y el progreso del puzzle.
- `H`: ocultar o mostrar la ayuda.

## Próximos avances

- Animación real de la rueda de cobre y del agua.
- Acumulación temporal y modo de captura en alta resolución.
- Texturas almacenadas como recursos del proyecto.
- Video de presentación y galería final.
