import { NextResponse } from 'next/server';
import { hardwareController } from '@/lib/hardware-control';

/**
 * GET /api/hardware/status
 * Check hardware availability and capabilities
 */
export async function GET() {
  try {
    const status = await hardwareController.initialize();

    return NextResponse.json(
      {
        success: true,
        hardwareAvailable: status.hasAccess,
        capabilities: status.features,
        message: status.message,
        operatingMode: status.hasAccess ? 'hardware-control' : 'demo-mode',
      },
      { status: 200 }
    );
  } catch (error) {
    console.error('[v0] Hardware status error:', error);
    return NextResponse.json(
      {
        success: false,
        hardwareAvailable: false,
        error: 'Failed to check hardware status',
        operatingMode: 'demo-mode',
      },
      { status: 500 }
    );
  }
}
