import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

interface ControlRequest {
  action: 'set-fan-mode' | 'set-profile' | 'get-capabilities';
  channel?: 'cpu' | 'gpu';
  mode?: 'auto' | 'manual' | 'maximum';
  duty?: number;
  profile?: 'quiet' | 'balanced' | 'performance' | 'g-mode';
}

interface ControlResponse {
  success: boolean;
  message: string;
  data?: any;
  error?: string;
}

async function executeDBusCommand(command: string): Promise<string> {
  try {
    const { stdout } = await execAsync(command, { timeout: 5000 });
    return stdout;
  } catch (error) {
    throw new Error(`D-Bus command failed: ${String(error)}`);
  }
}

async function setFanMode(
  channel: 'cpu' | 'gpu',
  mode: 'auto' | 'manual' | 'maximum',
  duty?: number
): Promise<ControlResponse> {
  try {
    let command = `dbus-send --system --dest=org.g15fanctl.Daemon1 `;
    command += `/org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetFanMode `;
    command += `string:${channel} string:${mode}`;

    if (mode === 'manual' && duty !== undefined) {
      // Ensure duty is between 40 and 100
      const validDuty = Math.max(40, Math.min(100, duty));
      command += ` byte:${validDuty}`;
    }

    await executeDBusCommand(command);

    return {
      success: true,
      message: `${channel.toUpperCase()} fan set to ${mode}${
        mode === 'manual' ? ` at ${duty}%` : ''
      }`,
    };
  } catch (error) {
    return {
      success: false,
      message: 'Failed to set fan mode',
      error: String(error),
    };
  }
}

async function setProfile(
  profile: 'quiet' | 'balanced' | 'performance' | 'g-mode'
): Promise<ControlResponse> {
  try {
    const command = `dbus-send --system --dest=org.g15fanctl.Daemon1 `;
    `org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.SetProfile string:${profile}`;

    await executeDBusCommand(command);

    return {
      success: true,
      message: `Thermal profile changed to ${profile}`,
    };
  } catch (error) {
    return {
      success: false,
      message: 'Failed to set profile',
      error: String(error),
    };
  }
}

async function getCapabilities(): Promise<ControlResponse> {
  try {
    const command = `dbus-send --system --print-reply --dest=org.g15fanctl.Daemon1 `;
    `/org/g15fanctl/Daemon1 org.g15fanctl.Daemon1.GetCapabilities`;

    const output = await executeDBusCommand(command);

    return {
      success: true,
      message: 'Capabilities retrieved',
      data: {
        manualFanControl: true,
        multiProfile: true,
        supportedProfiles: ['quiet', 'balanced', 'performance', 'g-mode'],
        supportedModes: ['auto', 'manual', 'maximum'],
      },
    };
  } catch (error) {
    return {
      success: false,
      message: 'Failed to get capabilities',
      error: String(error),
    };
  }
}

export async function POST(request: Request) {
  try {
    const body: ControlRequest = await request.json();
    let response: ControlResponse;

    switch (body.action) {
      case 'set-fan-mode':
        if (!body.channel || !body.mode) {
          return Response.json(
            {
              success: false,
              message: 'Missing channel or mode parameter',
            },
            { status: 400 }
          );
        }
        response = await setFanMode(body.channel, body.mode, body.duty);
        break;

      case 'set-profile':
        if (!body.profile) {
          return Response.json(
            {
              success: false,
              message: 'Missing profile parameter',
            },
            { status: 400 }
          );
        }
        response = await setProfile(body.profile);
        break;

      case 'get-capabilities':
        response = await getCapabilities();
        break;

      default:
        return Response.json(
          {
            success: false,
            message: `Unknown action: ${body.action}`,
          },
          { status: 400 }
        );
    }

    const statusCode = response.success ? 200 : 503;
    return Response.json(response, { status: statusCode });
  } catch (error) {
    return Response.json(
      {
        success: false,
        message: 'Request processing failed',
        error: String(error),
      },
      { status: 500 }
    );
  }
}
