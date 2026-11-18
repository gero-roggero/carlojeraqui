//! Captura de una sola ventana con Windows Graphics Capture (Windows 10 1903 o más nuevo): se ve la
//! ventana aunque esté tapada por otras o en otro monitor. La imagen queda en la GPU, como la del monitor.

use anyhow::{anyhow, bail, Result};
use windows::core::Interface;
use windows::Graphics::Capture::{Direct3D11CaptureFrame, Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Graphics::SizeInt32;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11Texture2D};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::System::WinRT::Direct3D11::{CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::Win32::UI::WindowsAndMessaging::{IsIconic, IsWindow};

const FORMAT: DirectXPixelFormat = DirectXPixelFormat::B8G8R8A8UIntNormalized;
/// Cuadros en espera: con dos, uno se copia mientras llega el siguiente.
const BUFFERS: i32 = 2;

pub struct WindowSource {
    hwnd: HWND,
    d3d: IDirect3DDevice,
    pool: Direct3D11CaptureFramePool,
    session: GraphicsCaptureSession,
    _item: GraphicsCaptureItem,
    size: SizeInt32,
}

impl WindowSource {
    pub fn new(device: &ID3D11Device, hwnd: HWND) -> Result<Self> {
        if !GraphicsCaptureSession::IsSupported().unwrap_or(false) {
            bail!("esta versión de Windows no puede grabar una sola ventana (hace falta Windows 10 1903 o más nuevo)");
        }
        unsafe {
            let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
            let item: GraphicsCaptureItem = interop.CreateForWindow(hwnd)?;
            let dxgi: IDXGIDevice = device.cast()?;
            let d3d: IDirect3DDevice = CreateDirect3D11DeviceFromDXGIDevice(&dxgi)?.cast()?;
            let size = item.Size()?;
            if size.Width <= 0 || size.Height <= 0 {
                bail!("la ventana no tiene tamaño (¿está minimizada?)");
            }
            let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(&d3d, FORMAT, BUFFERS, size)?;
            let session = pool.CreateCaptureSession(&item)?;
            let _ = session.SetIsCursorCaptureEnabled(true);
            // Windows 11: sin el borde amarillo alrededor de la ventana.
            let _ = session.SetIsBorderRequired(false);
            session.StartCapture()?;
            Ok(Self { hwnd, d3d, pool, session, _item: item, size })
        }
    }

    /// Tamaño de la ventana al empezar (par, como lo pide H.264).
    pub fn size(&self) -> (u32, u32) {
        (self.size.Width.max(2) as u32 & !1, self.size.Height.max(2) as u32 & !1)
    }

    /// Cuadro nuevo de la ventana, si llegó uno. Hay que copiar la textura antes de soltar el cuadro.
    pub fn next(&mut self) -> Result<Option<(Direct3D11CaptureFrame, ID3D11Texture2D)>> {
        unsafe {
            if !IsWindow(self.hwnd).as_bool() {
                bail!("se cerró la ventana que se estaba grabando");
            }
            if IsIconic(self.hwnd).as_bool() {
                return Ok(None); // minimizada: queda el último cuadro
            }
        }
        // Sin cuadro nuevo, Windows devuelve nada (y windows-rs, un error).
        let Ok(frame) = self.pool.TryGetNextFrame() else { return Ok(None) };
        let size = frame.ContentSize()?;
        if size.Width > 0 && size.Height > 0 && (size.Width != self.size.Width || size.Height != self.size.Height) {
            // La ventana cambió de tamaño: los próximos cuadros vienen al tamaño nuevo.
            self.size = size;
            self.pool.Recreate(&self.d3d, FORMAT, BUFFERS, size)?;
        }
        let access: IDirect3DDxgiInterfaceAccess = frame.Surface()?.cast()?;
        let tex: ID3D11Texture2D = unsafe { access.GetInterface()? };
        Ok(Some((frame, tex)))
    }
}

impl Drop for WindowSource {
    fn drop(&mut self) {
        let _ = self.session.Close();
        let _ = self.pool.Close();
    }
}

/// Texto de error para cuando la ventana elegida no está abierta.
pub fn not_open(label: &str) -> anyhow::Error {
    anyhow!("no está abierta la ventana elegida ({label}). Abrila y Carlojeraqui empieza a grabarla solo.")
}
