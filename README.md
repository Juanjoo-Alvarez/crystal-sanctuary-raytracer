# El Santuario del Cristal

**Proyecto 2 — Diorama con Raytracing**  
**Autor:** Juan Jose Rivas Alvarez — Carnet 24856

Diorama interactivo inspirado en la lectura visual de las maquetas de aventura. Está construido en Rust y utiliza raylib únicamente para crear la ventana, leer los controles y presentar el framebuffer. La imagen 3D se calcula con un raytracer CPU propio.

## Video demostrativo

[▶ Ver la campaña completa: Colinas, Mansión Boo y Ruinas de la Marea](videos/SantuarioDelCristal_Demo.mp4)

La demostración fue capturada en 1280×720 y 30 FPS utilizando el modo cinematográfico automático del proyecto. La mezcla cambia de banda sonora con cada mundo e incluye los efectos sintetizados de inicio, fragmentos, cristales y victoria.

## Estado actual

- Intersección analítica rayo–caja mediante el método *slab*.
- Cámara orbital, zoom y seguimiento automático del personaje.
- Framebuffer fijo de 512×288 con cuadros completos, sin barrido entrelazado ni cambios de resolución.
- Personaje voxel de explorador inspirado en Captain Toad, con sombrero, linterna emisiva, mochila, animación de caminata y cámara de seguimiento.
- Escala compacta del personaje ajustada a los pasillos y obstáculos del diorama.
- Colisiones AABB con columnas, ruinas, agua, mecanismos y bordes del diorama.
- Puente ensanchado con descansos abiertos y acceso transitable por peldaños, sin necesidad de salto.
- Rutas de objetivos despejadas y composición por terrazas inspirada en los niveles de Captain Toad.
- Puzzle de tres fragmentos emisivos y pulsantes: al recogerlos se puede activar el cristal central, cambiar su energía y transformar la iluminación de la escena.
- Objetivo guiado mediante HUD, indicadores de progreso y mensajes de descubrimiento.
- Mini campaña de tres niveles con arquitectura y recorridos propios: las Colinas usan terrazas, puente y ruinas; la Mansión de los Boo conecta cementerio, cripta elevada e interior; y las Ruinas de la Marea separan muelle y arrecife mediante un canal que obliga a cruzar un puente estrecho.
- Dioramas ampliados con exploración vertical: cumbre boscosa, campanario embrujado y acantilado con faro esconden un fragmento fuera de la ruta principal.
- Un elevador temático sube y baja en cada mundo; Tod debe encontrarlo, esperar la plataforma, viajar con ella y regresar después de explorar la zona alta.
- Portada animada con el diorama giratorio, entrada directa al juego y modo demostración.
- Carteles animados con nombre, relato y color propio para presentar cada diorama.
- Secuencia cinematográfica al despertar cada cristal: cámara orbital, destello y explosión de energía.
- Enemigos voxel animados por nivel: Goomba, Shy Guy, Biddybud, Boo, Peepa, Innertube Goomba y Stingby.
- Comportamiento clásico de los Boo: avanzan fuera de cámara y se congelan cuando son observados.
- Sistema de tres vidas, daño por contacto, invulnerabilidad temporal, Game Over y reinicio de nivel.
- Victoria por activación del cristal, transición automática y pantalla final con tiempo, vidas restantes y calificación de una a tres estrellas.
- Modo cinemático con ruta automática, cámara suave, interfaz oculta y raytracing recursivo para grabar video.
- Escena modular formada exclusivamente por bloques.
- Santuario ambientado con árboles voxelados, arbustos, faroles, un arco derruido y mampostería dispersa.
- Agua, cascada y cristales con variación animada calculada dentro del material.
- Seis materiales principales procedurales con parámetros independientes, además de materiales del personaje, vegetación y energía.
- Sombras, iluminación especular, niebla atmosférica, viñeta y tone mapping cinematográfico ACES.
- Shader de postprocesado GPU con bloom selectivo, color grading cálido/turquesa y viñeta suave.
- Partículas temáticas superpuestas: hojas en las colinas, espíritus en la mansión, burbujas bajo la marea y destellos al recoger fragmentos.
- Banda sonora OGG integrada, reproducida en streaming y en bucle para no depender de la carpeta de ejecución.
- Director musical por contexto: tema exclusivo para la portada, las colinas, la Mansión de los Boo y la playa, con cambio automático entre escenas.
- Efectos originales de inicio, fragmento, cristal, daño y victoria sintetizados por código, sin material de terceros.
- Interfaz rediseñada con la tipografía abierta Fredoka SemiBold y textos breves de estilo aventurero.
- Reflexión recursiva en cobre y cristal.
- Refracción con índice de refracción y efecto Fresnel.
- Tres skyboxes procedurales y color grading temático: atardecer, noche embrujada y ambiente acuático.
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

La aplicación mantiene siempre un framebuffer de 512×288 y presenta únicamente cuadros completos para evitar mareo o mezcla entre posiciones de cámara. El modo interactivo usa una muestra directa sin rebotes para mantener la respuesta; el modo cinemático conserva la resolución y activa un rebote recursivo para reflexión y refracción.

Para medir el rendimiento interactivo del raytracer sin depender de la interfaz:

```bash
cargo run --release -- --benchmark
```

Para generar una captura de 960×540 sin abrir la ventana:

```bash
cargo run --release -- --render-preview
```

Para ejecutar automáticamente toda la campaña y cerrar la ventana después de la pantalla final:

```bash
cargo run --release -- --record-demo
```

## Controles

- `ENTER`: comenzar la aventura desde la portada.
- `WASD`: mover a Toad en relación con la cámara.
- Arrastrar con el botón izquierdo: orbitar la cámara.
- Rueda del mouse: acercar o alejar.
- Flechas izquierda/derecha: orbitar horizontalmente.
- Flechas arriba/abajo: cambiar la elevación de la cámara.
- `C`: iniciar la demostración desde la portada o alternar el modo cinemático durante la campaña.
- `R`: reiniciar el nivel actual, las vidas y el progreso del puzzle.
- `P`: activar o desactivar el shader de postprocesado para comparar imagen y rendimiento.
- `M`: pausar o reanudar la música del nivel.

La tipografía Fredoka se distribuye bajo la SIL Open Font License; su licencia se encuentra en `assets/fonts/OFL.txt`.

## Próximos avances

- Animación real de la rueda de cobre y del agua.
- Acumulación temporal y modo de captura en alta resolución.
- Texturas almacenadas como recursos del proyecto.
- Galería final de capturas.
