test:
	./test.sh

run:
	./run.sh '$(arg)'
	
clean:
	rm -f *.o *~ tmp*

.PHONY: test clean