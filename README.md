# El Santuario del Cristal

**Proyecto 2 — Diorama con Raytracing**  
**Autor:** Juan Jose Rivas Alvarez — Carnet 24856

Diorama interactivo inspirado en la lectura visual de las maquetas de aventura. Está construido en Rust y utiliza raylib únicamente para crear la ventana, leer los controles y presentar el framebuffer. La imagen 3D se calcula con un raytracer CPU propio.

## Estado actual

- Intersección analítica rayo–caja mediante el método *slab*.
- Cámara orbital, zoom y rotación automática.
- Resolución dinámica: respuesta rápida en movimiento y refinado automático al detenerse.
- Escena modular formada exclusivamente por bloques.
- Cinco materiales procedurales con parámetros independientes.
- Sombras, iluminación especular y corrección gamma.
- Reflexión recursiva en cobre y cristal.
- Refracción con índice de refracción y efecto Fresnel.
- Skybox procedural de atardecer.
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

La aplicación comienza con la rotación pausada. Mientras se mueve la cámara usa un framebuffer de 400×225 para responder con rapidez; después de 0.3 segundos sin movimiento cambia automáticamente a 800×450, dos muestras por píxel y cuatro rebotes. La etiqueta inferior indica `INTERACTIVO` o `CALIDAD`.

Para generar una captura de 960×540 sin abrir la ventana:

```bash
cargo run --release -- --render-preview
```

## Controles

- Arrastrar con el botón izquierdo: orbitar la cámara.
- Rueda del mouse: acercar o alejar.
- `Espacio`: activar o detener la rotación automática.
- `R`: restaurar la cámara.
- `H`: ocultar o mostrar la ayuda.

## Próximos avances

- Animación real de la rueda de cobre y del agua.
- Mayor variedad de vegetación y arquitectura.
- Acumulación temporal y modo de captura en alta resolución.
- Texturas almacenadas como recursos del proyecto.
- Video de presentación y galería final.
