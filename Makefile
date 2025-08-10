test:
	./scripts/test_fn.sh '$(debug)'

test-in-main:
	./scripts/test.sh '$(option)'

run:
	./scripts/run.sh '$(arg)'
	
clean:
	rm -f *.o *~ tmp*

.PHONY: test clean run
