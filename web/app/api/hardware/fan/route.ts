import { NextRequest, NextResponse } from 'next/server';
import { hardwareController } from '@/lib/hardware-control';

interface FanRequest {
  channel: 'cpu' | 'gpu';
  mode?: 'auto' | 'manual' | 'maximum';
  dutyCycle?: number;
}

/**
 * POST /api/hardware/fan
 * Control fan mode and duty cycle
 * 
 * Request body:
 * {
 *   "channel": "cpu" | "gpu",
 *   "mode": "auto" | "manual" | "maximum",
 *   "dutyCycle": 40-100 (if mode is manual)
 * }
 */
export async function POST(request: NextRequest) {
  try {
    const body: FanRequest = await request.json();

    // Validate input
    if (!body.channel || !['cpu', 'gpu'].includes(body.channel)) {
      return NextResponse.json(
        { success: false, error: 'Invalid channel' },
        { status: 400 }
      );
    }

    if (!body.mode || !['auto', 'manual', 'maximum'].includes(body.mode)) {
      return NextResponse.json(
        { success: false, error: 'Invalid mode' },
        { status: 400 }
      );
    }

    // Handle different modes
    let result = false;
    let message = '';

    switch (body.mode) {
      case 'auto':
        result = await hardwareController.setFanMode(body.channel, 'auto');
        message = `${body.channel.toUpperCase()} fan set to AUTO mode`;
        break;

      case 'manual':
        if (!body.dutyCycle || body.dutyCycle < 40 || body.dutyCycle > 100) {
          return NextResponse.json(
            { success: false, error: 'dutyCycle must be 40-100' },
            { status: 400 }
          );
        }
        result = await hardwareController.setFanDuty(body.channel, body.dutyCycle);
        message = `${body.channel.toUpperCase()} fan set to ${body.dutyCycle}% duty cycle`;
        break;

      case 'maximum':
        result = await hardwareController.setFanDuty(body.channel, 100);
        message = `${body.channel.toUpperCase()} fan set to MAXIMUM (100%)`;
        break;
    }

    if (result) {
      return NextResponse.json(
        {
          success: true,
          message,
          channel: body.channel,
          mode: body.mode,
          dutyCycle: body.dutyCycle || (body.mode === 'maximum' ? 100 : undefined),
        },
        { status: 200 }
      );
    } else {
      return NextResponse.json(
        {
          success: false,
          error: 'Failed to control fan',
          message: 'Could not apply fan settings to hardware',
        },
        { status: 500 }
      );
    }
  } catch (error) {
    console.error('[v0] Fan control error:', error);
    return NextResponse.json(
      {
        success: false,
        error: 'Fan control failed',
        message: String(error),
      },
      { status: 500 }
    );
  }
}

/**
 * GET /api/hardware/fan?channel=cpu
 * Get current fan mode and status
 */
export async function GET(request: NextRequest) {
  try {
    const channel = request.nextUrl.searchParams.get('channel') as 'cpu' | 'gpu' || 'cpu';

    const mode = await hardwareController.getFanMode(channel);
    const rpm = await hardwareController.readFanRPM(channel);

    return NextResponse.json(
      {
        success: true,
        channel,
        mode,
        rpm,
        timestamp: new Date().toISOString(),
      },
      { status: 200 }
    );
  } catch (error) {
    console.error('[v0] Fan status error:', error);
    return NextResponse.json(
      {
        success: false,
        error: 'Failed to read fan status',
        message: String(error),
      },
      { status: 500 }
    );
  }
}
