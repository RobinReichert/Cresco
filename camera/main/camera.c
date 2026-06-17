#include "camera.h"
#include "esp_camera.h"
#include "sensor.h"

#define CAM_PIN_PWDN -1
#define CAM_PIN_RESET -1
#define CAM_PIN_VSYNC 38
#define CAM_PIN_HREF 47
#define CAM_PIN_PCLK 13
#define CAM_PIN_XCLK 10
#define CAM_PIN_SIOD 40
#define CAM_PIN_SIOC 39
#define CAM_PIN_D0 15
#define CAM_PIN_D1 17
#define CAM_PIN_D2 18
#define CAM_PIN_D3 16
#define CAM_PIN_D4 14
#define CAM_PIN_D5 12
#define CAM_PIN_D6 11
#define CAM_PIN_D7 48

static camera_fb_t *curr_fb = NULL;

static camera_config_t camera_config = {
    .pin_pwdn = CAM_PIN_PWDN,
    .pin_reset = CAM_PIN_RESET,
    .pin_xclk = CAM_PIN_XCLK,
    .pin_sccb_sda = CAM_PIN_SIOD,
    .pin_sccb_scl = CAM_PIN_SIOC,
    .pin_d7 = CAM_PIN_D7,
    .pin_d6 = CAM_PIN_D6,
    .pin_d5 = CAM_PIN_D5,
    .pin_d4 = CAM_PIN_D4,
    .pin_d3 = CAM_PIN_D3,
    .pin_d2 = CAM_PIN_D2,
    .pin_d1 = CAM_PIN_D1,
    .pin_d0 = CAM_PIN_D0,
    .pin_vsync = CAM_PIN_VSYNC,
    .pin_href = CAM_PIN_HREF,
    .pin_pclk = CAM_PIN_PCLK,
    .xclk_freq_hz = 20000000,
    .ledc_timer = LEDC_TIMER_0,
    .ledc_channel = LEDC_CHANNEL_0,
    .pixel_format = PIXFORMAT_JPEG,
    .frame_size = FRAMESIZE_HD,
    .jpeg_quality = 12,
    .fb_count = 1,
    .fb_location = CAMERA_FB_IN_PSRAM,
    .grab_mode = CAMERA_GRAB_WHEN_EMPTY,
};

esp_err_t init_camera() {
  esp_err_t err = esp_camera_init(&camera_config);
  if (err != ESP_OK) {
    return err;
  }
  sensor_t *s = esp_camera_sensor_get();
  s->set_vflip(s, 1);
  s->set_brightness(s, 1);
  s->set_saturation(s, -2);
  return ESP_OK;
}

esp_err_t capture_image(const uint8_t **buffer, size_t *len) {
  if (curr_fb) {
    return ESP_ERR_INVALID_STATE;
  }
  curr_fb = esp_camera_fb_get();
  if (!curr_fb) {
    return ESP_FAIL;
  }
  *buffer = curr_fb->buf;
  *len = curr_fb->len;
  return ESP_OK;
}

esp_err_t release_image() {
  if (!curr_fb) {
    return ESP_ERR_INVALID_STATE;
  }
  esp_camera_fb_return(curr_fb);
  return ESP_OK;
}
