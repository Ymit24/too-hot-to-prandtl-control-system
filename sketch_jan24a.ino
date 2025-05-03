#include <Arduino_FreeRTOS.h>
#include <queue.h>
#include <semphr.h>

void MyCoolTask(void *pvParameters);
void TaskReadAnalog(void *pvParameters);
void TaskSendSerialMessage(void *pvParameters);
void TaskRecvSerialMessage(void *pvParameters);

enum MessageType : uint8_t {
  SendSensors,
  RecvControls,
  SendLogLine
};

struct SendSensorsMsg {
  uint8_t sense1;
};

struct SendLogLineMsg {
  uint8_t len;
  char line[];
};

struct RecvControlsMsg {
  bool led_state;
};

struct __attribute__((packed)) MessageHeader {
  uint8_t type;
  uint8_t len;
  uint8_t data[];
};

struct IntegratedMessage {
  MessageType type;
  union {
    SendSensorsMsg send_sensor;
    RecvControlsMsg recv_control;
    SendLogLineMsg send_log;
  } data;
};

QueueHandle_t xAnalogValueToPrintQueue;
QueueHandle_t xMessageSendQueue, xMessageRecvQueue;

SemaphoreHandle_t xSerialMutex;

#define TASK_SEND_STACK_SIZE 128
#define TASK_RECV_STACK_SIZE 128
#define TASK_COOL_STACK_SIZE 64
#define TASK_RA_STACK_SIZE 128

StaticTask_t xTaskSendBuffer;
StaticTask_t xTaskCoolBuffer;
StaticTask_t xTaskRecvBuffer;
StaticTask_t xTaskRABuffer;

StackType_t xStackSend[TASK_SEND_STACK_SIZE];
StackType_t xStackRecv[TASK_RECV_STACK_SIZE];
StackType_t xStackCool[TASK_COOL_STACK_SIZE];
StackType_t xStackRA[TASK_RA_STACK_SIZE];

#define QUEUE_SEND_LENGTH 1
#define QUEUE_SEND_ITEMSIZE sizeof(MessageHeader)

#define QUEUE_RECV_LENGTH 1
#define QUEUE_RECV_ITEMSIZE sizeof(IntegratedMessage)

static StaticQueue_t xMessageSendQueueStruct;
static StaticQueue_t xMessageRecvQueueStruct;

static uint8_t ucMessageSendQueueStorage[QUEUE_SEND_LENGTH * QUEUE_SEND_ITEMSIZE];
static uint8_t ucMessageRecvQueueStorage[QUEUE_RECV_LENGTH * QUEUE_RECV_ITEMSIZE];

static StaticSemaphore_t xSerialMutexBuffer;

void setup() {
  Serial.begin(9600);
  // Serial.println("Going to boot up the program.");

  xMessageSendQueue = xQueueCreateStatic(
    QUEUE_SEND_LENGTH,
    QUEUE_SEND_ITEMSIZE,
    ucMessageSendQueueStorage,
    &xMessageSendQueueStruct);
  xMessageRecvQueue = xQueueCreateStatic(
    QUEUE_RECV_LENGTH,
    QUEUE_RECV_ITEMSIZE,
    ucMessageRecvQueueStorage,
    &xMessageRecvQueueStruct);

  xSerialMutex = xSemaphoreCreateMutexStatic(&xSerialMutexBuffer);

  if (xMessageRecvQueue == NULL || xMessageSendQueue == NULL || xSerialMutex == NULL) {
    Serial.println("Failed to create either a queue or semaphore.");
    return;
  }

  xTaskCreateStatic(
    MyCoolTask,
    "MyCoolTask",
    TASK_COOL_STACK_SIZE,
    NULL,
    2,
    xStackCool,
    &xTaskCoolBuffer);


  xTaskCreateStatic(
    TaskReadAnalog,
    "TaskReadAnalog",
    TASK_RA_STACK_SIZE,
    NULL,
    2,
    xStackRA,
    &xTaskRABuffer);

  xTaskCreateStatic(
    TaskSendSerialMessage,
    "TaskSendSerialMessage",
    TASK_SEND_STACK_SIZE,
    NULL,
    2,
    xStackSend,
    &xTaskSendBuffer);

  xTaskCreateStatic(
    TaskRecvSerialMessage,
    "TaskRecvSerialMessage",
    TASK_RECV_STACK_SIZE,
    NULL,
    2,
    xStackRecv,
    &xTaskRecvBuffer);
}

void loop() {}

void TaskSendSerialMessage(void *pvParameters) {
  static MessageHeader header;
  for (;;) {
    if (xQueueReceive(xMessageSendQueue, &header, (TickType_t)3) == pdPASS) {
      size_t packet_size = sizeof(MessageHeader) + header.len;
      auto buffer = reinterpret_cast<const uint8_t *>(&header);

      Serial.print("Data2:");
      Serial.write(buffer, packet_size);
      Serial.println();

      Serial.available()

      if (xSemaphoreTake(xSerialMutex, 10) == pdTRUE) {
        // Serial.print(F("Packet Size: "));
        // Serial.println(packet_size);
        xSemaphoreGive(xSerialMutex);
      }
    }
    vTaskDelay(10);
  }
}

// void TaskSendSerialMessage2(void *pvParameters) {
//   static IntegratedMessage msg_to_send;
//   for (;;) {
//     if (xQueueReceive(xMessageSendQueue, &msg_to_send, (TickType_t)3) == pdPASS) {
//       size_t msg_size = sizeof(IntegratedMessage);
//       auto buffer = reinterpret_cast<const uint8_t *>(&msg_size);

//       // Serial.print("Data2:");
       // Serial.print(msg_size);
//       // Serial.println();

//       if (xSemaphoreTake(xSerialMutex, 10) == pdTRUE) {
//         Serial.write(buffer, msg_size);
//         xSemaphoreGive(xSerialMutex);
//       }
//     }
//     vTaskDelay(10);
//   }
// }

void TaskRecvSerialMessage(void *pvParameters) {
  static IntegratedMessage recv_msg;
  static bool did_receive = false;
  for (;;) {
    if (xSemaphoreTake(xSerialMutex, 10) == pdTRUE) {
      if (Serial.available() >= sizeof(IntegratedMessage)) {
        Serial.readBytes((char *)&recv_msg, sizeof(IntegratedMessage));
        did_receive = true;
      }
      xSemaphoreGive(xSerialMutex);
    }
    if (did_receive) {
      while (xQueueSend(xMessageRecvQueue, &recv_msg, (TickType_t)20) != pdPASS) {
        vTaskDelay(1);
      }
    }
    vTaskDelay(1);
  }
}

void MyCoolTask(void *pvParameters) {
  pinMode(LED_BUILTIN, OUTPUT);
  for (;;) {
    // IntegratedMessage msg;
    // if (xQueueReceive(xMessageRecvQueue, &msg, portMAX_DELAY) == pdTRUE) {
    //   digitalWrite(LED_BUILTIN, msg.data.recv_control.led_state ? HIGH : LOW);
    // }
    vTaskDelay(10);
  }
}

void TaskReadAnalog(void *pvParameters) {
  static uint16_t value;
  static SendLogLineMsg data = { 7, "AAAAAA" };
  MessageHeader msg;

  pinMode(A4, INPUT);
  value = analogRead(A4);

  msg.type = 2;
  msg.len = sizeof(SendLogLineMsg) + data.len;
  memcpy(&msg.data, &data, sizeof(SendLogLineMsg) + data.len);

  // strcpy(msg.data.send_log.line, "abcdefg");

  for (;;) {
    xQueueSend(xMessageSendQueue, &msg, portMAX_DELAY);

    vTaskDelay(250 / portTICK_PERIOD_MS);
  }
}
