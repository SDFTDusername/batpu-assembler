#define MEM_ADDR r1
#define MEM_VAL r2

#define PLR_X r3
#define PLR_Y r4

main:
  ldi PLR_X 15
  ldi PLR_Y 15

loop:
  cal step
  cal render
  jmp loop

step:
  // Load controller data
  ldi r1 CONTROLLER
  lod r1 r6 0
  
  // Left movement
  ldi r7 0b0000_0001
  ldi r8 -1
  ldi r9 0
  cal check_movement
  
  // Right movement
  ldi r7 0b0000_0100
  ldi r8 1
  cal check_movement
  
  // Top movement
  ldi r7 0b0000_0010
  ldi r8 0
  ldi r9 -1
  cal check_movement
  
  // Bottom movement
  ldi r7 0b0000_1000
  ldi r9 1
  cal check_movement
  
  ret

check_movement:
  // Check if button is pressed
  and r6 r7 r0
  brh notzero check_start
  
  // Move player if button is pressed
  jmp check_end
  check_start:
    add PLR_X r8 PLR_X
    add PLR_Y r9 PLR_Y
  check_end:
  
  ret

render:
  // Clear screen buffer
  ldi MEM_ADDR SCR_CLR
  str MEM_ADDR r0 0
  
  // Draw player
  
  ldi MEM_ADDR SCR_PIX_X
  str MEM_ADDR PLR_X 0
  
  ldi MEM_ADDR SCR_PIX_Y
  str MEM_ADDR PLR_Y 0
  
  ldi MEM_ADDR SCR_DRAW_PIX
  str MEM_ADDR r0 0
  
  // Push screen buffer
  ldi MEM_ADDR SCR_PUSH
  str MEM_ADDR r0 0
  
  ret