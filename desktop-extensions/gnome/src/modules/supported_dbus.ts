declare const global: any, imports: any;
//@ts-ignore
const Me = imports.misc.extensionUtils.getCurrentExtension();

import * as Resources from './resources';

import * as Platform from '../bindings/platform';
import * as Aura from '../bindings/aura';

const { Gio } = imports.gi;

export class Supported {
  supportedProxy: any = null; // type: Gio.DbusProxy (donno how to add)
  connectedSupported: boolean = false;

  // False,
  // (True,),
  // (True, True),
  //   ('X19b6',
  //   True,
  //   ['Static',
  //   'Breathe',
  //   'Strobe',
  //   'Rainbow',
  //   'Star',
  //   'Rain',
  //   'Highlight',
  //   'Laser',
  //   'Ripple',
  //   'Pulse',
  //   'Comet',
  //   'Flash'],
  //   [],
  //   2),
  // (False, True, True, True, False, True)

  supported: Platform.SupportedFunctions = {
    anime_ctrl: false,
    charge_ctrl: {
      charge_level_set: false
    },
    platform_profile: {
      platform_profile: false,
      fan_curves: false
    },
    keyboard_led: {
      dev_id: Aura.AuraDevice.Unknown,
      brightness: false,
      basic_modes: [],
      basic_zones: [],
      advanced_type: Platform.AdvancedAura.None
    },
    rog_bios_ctrl: {
      post_sound: false,
      gpu_mux: false,
      panel_overdrive: false,
      dgpu_disable: false,
      egpu_enable: false,
      mini_led_mode: false
    }
  };

  constructor() {
    // nothing for now
  }

  public getSupported() {
    if (this.isRunning()) {
      try {
        let _supportedAttributes = this.supportedProxy.SupportedFunctionsSync();
        if (_supportedAttributes.length > 0) {
          let valueString: string = '';

          for (const [_key, value] of Object.entries(_supportedAttributes)) {
            //@ts-ignore
            valueString = value.toString();

            switch (parseInt(_key)) {
              case 0:
                this.supported.anime_ctrl = (valueString == 'true' ? true : false);
                break;

              case 1:
                this.supported.charge_ctrl.charge_level_set = (valueString == 'true' ? true : false);
                break;

              case 2:
                let platformArray = valueString.split(',');
                this.supported.platform_profile.fan_curves = (platformArray[0] == 'true' ? true : false);
                this.supported.platform_profile.platform_profile = (platformArray[1] == 'true' ? true : false);
                break;

              case 3:
                let ledArray = valueString.split(',');
                // let t: keyof typeof AuraDevice = ledArray[0]; // can't conevert
                this.supported.keyboard_led.dev_id = Aura.AuraDevice[ledArray[0] as Aura.AuraDevice];
                this.supported.keyboard_led.brightness = (ledArray[1] == 'true' ? true : false);
                this.supported.keyboard_led.basic_modes = ledArray[2].split(',').map(function (value) {
                  return Aura.AuraModeNum[value as Aura.AuraModeNum]
                });
                this.supported.keyboard_led.basic_zones = ledArray[3].split(',').map(function (value) {
                  return Aura.AuraZone[value as Aura.AuraZone]
                });
                this.supported.keyboard_led.advanced_type = Platform.AdvancedAura[ledArray[4] as Platform.AdvancedAura];
                break;

              case 4:
                let biosArray = valueString.split(',');
                this.supported.rog_bios_ctrl.post_sound = (biosArray[0] == 'true' ? true : false);
                this.supported.rog_bios_ctrl.gpu_mux = (biosArray[1] == 'true' ? true : false);
                this.supported.rog_bios_ctrl.panel_overdrive = (biosArray[2] == 'true' ? true : false);
                this.supported.rog_bios_ctrl.dgpu_disable = (biosArray[3] == 'true' ? true : false);
                this.supported.rog_bios_ctrl.egpu_enable = (biosArray[4] == 'true' ? true : false);
                this.supported.rog_bios_ctrl.mini_led_mode = (biosArray[5] == 'true' ? true : false);
                break;

              default:
                break;
            }
          }
        }
      } catch (e) {
        //@ts-ignore
        log(`Failed to fetch supported functionalities`, e);
      }
    }
  }

  isRunning(): boolean {
    return this.connectedSupported;
  }

  async start() {
    try {
      // creating the proxy
      let xml = Resources.File.DBus('org-asuslinux-supported-4');
      this.supportedProxy = new Gio.DBusProxy.makeProxyWrapper(xml)(
        Gio.DBus.system,
        'org.asuslinux.Daemon',
        '/org/asuslinux/Supported'
      );

      this.connectedSupported = true;

      this.getSupported();
      //@ts-ignore
      log(`Supported Daemon client started successfully.`);
    } catch (e) {
      //@ts-ignore
      log(`Supported DBus initialization failed!`, e);
    }
  }

  stop() {
    //@ts-ignore
    log(`Stopping Supported DBus client...`);

    if (this.isRunning()) {
      this.connectedSupported = false;
      this.supportedProxy = null;
    }
  }
}