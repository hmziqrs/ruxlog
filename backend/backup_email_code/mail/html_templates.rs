pub fn email_otp_html(code: &str) -> String {
    format!(r#"
    <!DOCTYPE html>
    <html lang="en">
      <body>
        <div
          style='background-color:#000000;color:#FFFFFF;font-family:"Iowan Old Style", "Palatino Linotype", "URW Palladio L", P052, serif;font-size:16px;font-weight:400;letter-spacing:0.15008px;line-height:1.5;margin:0;padding:32px 0;min-height:100%;width:100%'
        >
          <table
            align="center"
            width="100%"
            style="margin:0 auto;max-width:600px;background-color:#000000"
            role="presentation"
            cellspacing="0"
            cellpadding="0"
            border="0"
          >
            <tbody>
              <tr style="width:100%">
                <td>
                  <div style="padding:24px 24px 24px 24px;text-align:center">
                    <img
                      alt=""
                      src="https://d1iiu589g39o6c.cloudfront.net/live/platforms/platform_A9wwKSL6EV6orh6f/images/wptemplateimage_jc7ZfPvdHJ6rtH1W/&amp;.png"
                      height="24"
                      style="height:24px;outline:none;border:none;text-decoration:none;vertical-align:middle;display:inline-block;max-width:100%"
                    />
                  </div>
                  <div
                    style="color:#ffffff;font-size:16px;font-weight:normal;text-align:center;padding:16px 24px 16px 24px"
                  >
                    Here is your one-time passcode:
                  </div>
                  <h1
                    style='font-weight:bold;text-align:center;margin:0;font-family:"Nimbus Mono PS", "Courier New", "Cutive Mono", monospace;font-size:32px;padding:16px 24px 16px 24px'
                  >
                    {}
                  </h1>
                  <div
                    style="color:#868686;font-size:16px;font-weight:normal;text-align:center;padding:16px 24px 16px 24px"
                  >
                    This code will expire in 3 hours.
                  </div>
                  <!-- <div
                    style="color:#868686;font-size:14px;font-weight:normal;text-align:center;padding:16px 24px 16px 24px"
                  >
                    Problems? Just reply to this email.
                  </div> -->
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </body>
    </html>
    "#, code).to_string()
}
