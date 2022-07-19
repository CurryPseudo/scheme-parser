(lambda (x) x)
(lambda (x y) x)
(lambda (x y) 
  (define z (x 3.57))
  (define y (z #t))
  (x -1235 z)
  y
)