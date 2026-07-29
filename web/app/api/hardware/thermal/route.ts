import { NextRequest, NextResponse } from 'next/server';
import { hardwareController } from '@/lib/hardware-control';

/**
 * GET /api/hardware/thermal
 * Returns current thermal data from hardware sensors
 */
export async function GET(request: NextRequest) {
  try {
    const thermalData = await hardwareController.getThermalData();

    return NextResponse.json(
      {
        success: true,
        data: thermalData,
        timestamp: new Date().toISOString(),
      },
      { status: 200 }
    );
  } catch (error) {
    console.error('[v0] Thermal API error:', error);
    return NextResponse.json(
      {
        success: false,
        error: 'Failed to read thermal data',
        message: String(error),
      },
      { status: 500 }
    );
  }
}
