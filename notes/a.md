```java
int quantity = 5;
float total = 3 * quantity;
byte newByte = 126;
int newInt = 0xa2eee;

/*
why? because the format string was set to %x where x stands for hex in lowercase.
*/

System.out.println(quantity);
System.out.println(total);
System.out.println(newByte);
System.out.printf("%x\n",newInt);
```