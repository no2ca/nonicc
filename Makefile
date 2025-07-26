test:
	./test.sh '$(option)'

run:
	./run.sh '$(arg)'
	
clean:
	rm -f *.o *~ tmp*

.PHONY: test clean run
